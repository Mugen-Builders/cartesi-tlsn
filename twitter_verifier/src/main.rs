use crabrolls::prelude::*;
use elliptic_curve::pkcs8::DecodePublicKey;
use ethabi::ethereum_types::U256;
use ethabi::Address;
use ethabi::Token;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::str;
use std::sync::Mutex;
use tlsn_core::proof::{SessionProof, TlsProof};

#[derive(Serialize)]
struct TwitterProfile {
    username: String,
    number_of_followers: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "metadata")]
enum InspectInput {
    #[serde(rename = "followers")]
    Followers { address: Address },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", content = "payload")]
enum AdvanceInput {
    ProofOfOwnership { username: String, proof: TlsProof },
    ProofOfTheNumberOfFollowers { username: String, proof: TlsProof },
}

struct TwitterApp {
    // Add a field to store internal state
    number_of_followers_state: Mutex<HashMap<Address, u64>>,
}

impl TwitterApp {
    fn new() -> Self {
        Self {
            number_of_followers_state: Mutex::new(HashMap::new()),
        }
    }
}

impl Application for TwitterApp {
    async fn advance(
        &self,
        env: &impl Environment,
        metadata: Metadata,
        payload: &[u8],
        deposit: Option<Deposit>,
    ) -> Result<FinishStatus, Box<dyn Error>> {
        let input: AdvanceInput = serde_json::from_slice(payload)?;

        match input {
            AdvanceInput::ProofOfOwnership { username, proof } => {
                // Verify the proof of ownership
                verify_account_ownership(username.clone(), proof)?;
                println!(
                    "Address {:?} proved ownership of the username {}",
                    metadata.sender, username
                );

                let abi_json = r#"
                [
                    {
                        "name": "safeMint",
                        "inputs": [
                            {
                                "internalType": "address",
                                "name": "to",
                                "type": "address"
                            },
                            {
                                "internalType": "uint256",
                                "name": "tokenId",
                                "type": "uint256"
                            },
                            {
                                "internalType": "string",
                                "name": "uri",
                                "type": "string"
                            }
                        ],
                        "outputs": [],
                        "type": "function"
                    }
                ]
                "#;

                let function_name = "safeMint";
                let mut token_id = U256::from(1);

                let params = vec![
                    Token::Address(metadata.sender),
                    Token::Uint(token_id),
                    Token::String(username),
                ];
                let payload = abi::encode::function_call(abi_json, function_name, params)
                    .expect("Failed to encode function call");
                env.send_voucher(
                    address!("0x1234567890123456789012345678901234567890"),
                    payload,
                )
                .await?;

                // Increment tokenId
                token_id += U256::from(1);
            }
            AdvanceInput::ProofOfTheNumberOfFollowers { username, proof } => {
                // Verify the proof of the number of followers
                let follower_count = verify_twitter_followers(username.clone(), proof)?;
                println!(
                    "Address {:?} proved the number of followers of the username {} is {}",
                    metadata.sender, username, follower_count.number_of_followers
                );

                {
                    let mut memory = self.number_of_followers_state.lock().unwrap();
                    memory.insert(metadata.sender, follower_count.number_of_followers);
                }

                // Send a notice with the number of followers
                env.send_notice(serde_json::to_vec(&follower_count)?)
                    .await?;
            }
        }

        Ok(FinishStatus::Accept)
    }

    async fn inspect(
        &self,
        env: &impl Environment,
        payload: &[u8],
    ) -> Result<FinishStatus, Box<dyn Error>> {
        let inspect = match serde_json::from_slice::<InspectInput>(payload) {
            Ok(inspect) => inspect,
            Err(e) => {
                println!("Error deserializing inspection request: {}", e);
                return Ok(FinishStatus::Reject);
            }
        };

        match inspect {
            InspectInput::Followers { address } => {
                let memory = self.number_of_followers_state.lock().unwrap();
                if let Some(&number_of_followers) = memory.get(&address) {
                    let res = serde_json::json!({
                        "number_of_followers": number_of_followers
                    });
                    env.send_report(serde_json::to_vec(&res)?).await?;
                } else {
                    let res = serde_json::json!({
                        "address": format!("{:?}", address),
                        "error": "Address not found"
                    });
                    env.send_report(serde_json::to_vec(&res)?).await?;
                }
            }
        }
        Ok(FinishStatus::Accept)
    }
}

#[async_std::main]
async fn main() {
    let app = TwitterApp::new();
    let options = RunOptions::default();
    if let Err(e) = Supervisor::run(app, options).await {
        eprintln!("Error: {}", e);
    }
}

/// Returns a Notary public key trusted by this Verifier
fn notary_pubkey() -> p256::PublicKey {
    let pem_file = str::from_utf8(include_bytes!("../notary.pub")).unwrap();
    p256::PublicKey::from_public_key_pem(pem_file).unwrap()
}

/// Verifies the proof of ownership of the username
fn verify_account_ownership(
    target_username: String,
    proof: TlsProof,
) -> Result<(), Box<dyn std::error::Error>> {
    let TlsProof {
        session,
        substrings,
    } = proof;

    session.verify_with_default_cert_verifier(notary_pubkey())?;

    let SessionProof { header, .. } = session;

    let (mut sent, mut recv) = substrings.verify(&header)?;

    sent.set_redacted(b'X');
    recv.set_redacted(b'X');

    check_prefix(
        String::from_utf8(sent.data().to_vec()).unwrap(),
        "GET https://api.x.com/1.1/account/settings.json".to_string(),
    )?;

    let auth_json_obj = deserialize_json(String::from_utf8(recv.data().to_vec()).unwrap())?;
    if let Some(screen_name) = auth_json_obj.get("screen_name").and_then(|x| x.as_str()) {
        if screen_name.to_lowercase() != target_username.to_lowercase() {
            return Err(Box::from(format!(
                "Incorrect username in ownership proof: {}",
                screen_name
            )));
        }
    } else {
        return Err(Box::from(format!(
            "Auth JSON object did not contain 'screen_name': {}",
            auth_json_obj
        )));
    }

    Ok(())
}

/// Verifies the proof of the number of followers
fn verify_twitter_followers(
    username: String,
    proof: TlsProof,
) -> Result<TwitterProfile, Box<dyn std::error::Error>> {
    let TlsProof {
        session,
        substrings,
    } = proof;

    session.verify_with_default_cert_verifier(notary_pubkey())?;

    let SessionProof { header, .. } = session;

    let (mut sent, mut recv) = substrings.verify(&header)?;

    sent.set_redacted(b'X');
    recv.set_redacted(b'X');

    check_prefix(
        String::from_utf8(sent.data().to_vec()).unwrap(),
        "GET https://x.com/i/api/graphql/".to_string(),
    )?;
    check_contains(
        String::from_utf8(sent.data().to_vec()).unwrap(),
        format!(
            "UserByScreenName?variables=%7B%22screen_name%22%3A%22{}",
            username
        ),
    )?;

    let user_json_obj = deserialize_json(String::from_utf8(recv.data().to_vec()).unwrap())?;
    let unwrapped_user_data = user_json_obj
        .get("data")
        .and_then(|x| x.get("user"))
        .and_then(|x| x.get("result"))
        .and_then(|x| x.get("legacy"))
        .ok_or("Failed to find user data")?;

    if let Some(screen_name) = unwrapped_user_data
        .get("screen_name")
        .and_then(|x| x.as_str())
    {
        if screen_name.to_lowercase() != username.to_lowercase() {
            return Err(Box::from(format!(
                "Incorrect username in follower proof: {}",
                screen_name
            )));
        }
    } else {
        return Err(Box::from(format!(
            "User JSON object did not contain 'screen_name': {}",
            user_json_obj
        )));
    }

    let follower_count = unwrapped_user_data
        .get("followers_count")
        .ok_or("Follower count not found")?
        .as_u64()
        .ok_or("Failed to convert to u64")?;

    Ok(TwitterProfile {
        username: username,
        number_of_followers: follower_count,
    })
}

/// Checks if the input string starts with the expected prefix
fn check_prefix(input_str: String, prefix_str: String) -> Result<(), Box<dyn std::error::Error>> {
    if input_str.trim_start().starts_with(&prefix_str) {
        Ok(())
    } else {
        Err(Box::from(format!(
            "The URL prefix '{}' was not found in the input header: '{}'",
            prefix_str, input_str
        )))
    }
}

/// Checks if the input string contains the expected substring
fn check_contains(input_str: String, sub_str: String) -> Result<(), Box<dyn std::error::Error>> {
    if input_str.contains(&sub_str) {
        Ok(())
    } else {
        Err(Box::from(format!(
            "The URL substring '{}' was not found in the input header: '{}'",
            sub_str, input_str
        )))
    }
}

/// Deserializes the last line of the input string as JSON
fn deserialize_json(input_str: String) -> Result<Value, Box<dyn std::error::Error>> {
    if let Some(last_line) = input_str.lines().last() {
        let json_value: Value = serde_json::from_str(last_line)?;
        Ok(json_value)
    } else {
        Err("No lines found in input".into())
    }
}

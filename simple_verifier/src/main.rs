use crabrolls::prelude::*;
use elliptic_curve::pkcs8::DecodePublicKey;
use ethabi::{Address};
use std::error::Error;
use std::{str, time::Duration};
use tlsn_core::proof::{SessionProof, TlsProof};

struct SimpleVerifierApp;

impl SimpleVerifierApp {
    fn new() -> Self {
        Self
    }
}

impl Application for SimpleVerifierApp {
    async fn advance(
        &self,
        env: &impl Environment,
        metadata: Metadata,
        payload: &[u8],
        deposit: Option<Deposit>,
    ) -> Result<FinishStatus, Box<dyn Error>> {
        match deposit {
            Some(Deposit::Ether { sender, amount }) => {
                println!(
                    "Received deposit of {} ether from {}",
                    units::wei::to_ether(amount),
                    sender
                );

                let balance = env.ether_balance(sender).await;
                println!(
                    "Current balance of sender: {} ether",
                    units::wei::to_ether(balance)
                );
            }
            Some(Deposit::ERC20 {
                sender,
                token,
                amount,
            }) => {
                println!(
                    "Received deposit of {} ERC20 tokens from {}",
                    amount, sender
                );

                let balance = env.erc20_balance(sender, token).await;
                println!("Current balance of sender's ERC20 tokens: {}", balance);
            }
            Some(Deposit::ERC721 { sender, token, id }) => {
                println!(
                    "Received ERC721 token with ID {} from {}, with token address {}",
                    id, sender, token
                );
            }
            Some(Deposit::ERC1155 {
                sender,
                token,
                ids_amounts,
            }) => {
                println!("Received ERC1155 deposit from {}", sender);

                for (id, _amount) in ids_amounts {
                    let balance = env.erc1155_balance(sender, token, id).await;
                    println!("Current balance of ERC1155 token ID {}: {}", id, balance);
                }
            }
            None => {
                let proof: TlsProof = serde_json::from_slice(payload)?;

                let TlsProof {
                    session,
                    substrings,
                } = proof;

                session
                    .verify_with_default_cert_verifier(notary_pubkey())
                    .map_err(|e| format!("Session verification failed: {:?}", e))?;

                let SessionProof {
                    header,
                    session_info,
                    ..
                } = session;

                let time =
                    Duration::from_secs(metadata.timestamp) + Duration::from_secs(header.time());

                let (mut sent, mut recv) = substrings
                    .verify(&header)
                    .map_err(|e| format!("Substrings verification failed: {:?}", e))?;

                sent.set_redacted(b'X');
                recv.set_redacted(b'X');

                println!("-------------------------------------------------------------------");
                println!(
                    "Successfully verified that the bytes below came from a session with {:?} at {:?}.",
                    session_info.server_name, time
                );
                println!(
                    "Note that the bytes which the Prover chose not to disclose are shown as X."
                );
                println!();
                println!("Bytes sent:");
                println!(
                    "{}",
                    String::from_utf8(sent.data().to_vec())
                        .unwrap_or_else(|_| "Invalid UTF-8".to_string())
                );
                println!();
                println!("Bytes received:");
                println!(
                    "{}",
                    String::from_utf8(recv.data().to_vec())
                        .unwrap_or_else(|_| "Invalid UTF-8".to_string())
                );
                println!("-------------------------------------------------------------------");

                let token_address = address!("0x92C6bcA388E99d6B304f1Af3c3Cd749Ff0b591e2");
                let vault = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
                env.erc20_withdraw(
                    vault,
                    token_address,
                    env.erc20_balance(vault, token_address).await,
                )
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
        println!(
            "Inspect method called with payload: {:?}",
            String::from_utf8_lossy(payload)
        );
        env.send_report(payload).await?;
        Ok(FinishStatus::Accept)
    }
}

#[async_std::main]
async fn main() {
    let app = SimpleVerifierApp::new();
    let options = RunOptions::default();
    if let Err(e) = Supervisor::run(app, options).await {
        eprintln!("Error: {}", e);
    }
}

/// Returns a Notary pubkey trusted by this Verifier
fn notary_pubkey() -> p256::PublicKey {
    let pem_file = str::from_utf8(include_bytes!("../notary.pub")).unwrap();
    p256::PublicKey::from_public_key_pem(pem_file).unwrap()
}

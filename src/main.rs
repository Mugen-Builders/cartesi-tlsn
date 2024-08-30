use std::error::Error;
use crabrolls::prelude::*;
use std::{str, time::Duration};
use elliptic_curve::pkcs8::DecodePublicKey;
use tlsn_core::proof::{SessionProof, TlsProof};

struct EchoApp;

impl EchoApp {
    fn new() -> Self {
        Self
    }
}

impl Application for EchoApp {
    async fn advance(
        &self,
        env: &impl Environment,
        metadata: Metadata,
        payload: &[u8],
        _deposit: Option<Deposit>,
    ) -> Result<FinishStatus, Box<dyn Error>> {
        let proof = std::fs::read_to_string("../simple_proof.json").unwrap();
        let proof: TlsProof = serde_json::from_str(proof.as_str()).unwrap();

        let TlsProof {
            // The session proof establishes the identity of the server and the commitments
            // to the TLS transcript.
            session,
            // The substrings proof proves select portions of the transcript, while redacting
            // anything the Prover chose not to disclose.
            substrings,
        } = proof;

        // Verify the session proof against the Notary's public key
        //
        // This verifies the identity of the server using a default certificate verifier which trusts
        // the root certificates from the `webpki-roots` crate.
        session
            .verify_with_default_cert_verifier(notary_pubkey())
            .unwrap();

        let SessionProof {
            // The session header that was signed by the Notary is a succinct commitment to the TLS transcript.
            header,
            // This is the session_info, which contains the server_name, that is checked against the
            // certificate chain shared in the TLS handshake.
            session_info,
            ..
        } = session;

        // The time at which the session was recorded
        let time = Duration::from_secs(metadata.timestamp) + Duration::from_secs(header.time());

        // Verify the substrings proof against the session header.
        //
        // This returns the redacted transcripts
        let (mut sent, mut recv) = substrings.verify(&header).unwrap();

        // Replace the bytes which the Prover chose not to disclose with 'X'
        sent.set_redacted(b'X');
        recv.set_redacted(b'X');

        println!("-------------------------------------------------------------------");
        println!(
            "Successfully verified that the bytes below came from a session with {:?} at {:?}.",
            session_info.server_name, time
        );
        println!("Note that the bytes which the Prover chose not to disclose are shown as X.");
        println!();
        println!("Bytes sent:");
        println!();
        print!("{}", String::from_utf8(sent.data().to_vec()).unwrap());
        println!();
        println!("Bytes received:");
        println!();
        println!("{}", String::from_utf8(recv.data().to_vec()).unwrap());
        println!("-------------------------------------------------------------------");

        println!(
            "Advance method called with payload: {:?}",
            String::from_utf8_lossy(payload)
        );

        env.send_notice(payload).await?;
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
    let app = EchoApp::new();
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
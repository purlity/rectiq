// rectiq-cli/src/divine/apply.rs
use crate::divine::manifest::StepManifest;
use crate::config::CliConfig;
use aes_gcm::{
    Aes256Gcm, KeySizeUser,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::error::Error;
use tracing::info;

#[derive(serde::Deserialize)]
struct KeyResponse {
    base64_key: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DivineFixEnvelope {
    pub v: u8,
    pub payload: String,    // base64: divine [nonce||ct||tag]
    pub nonce_hint: String, // index hint
    pub aad: String,
}

#[derive(serde::Serialize)]
struct RevealEnvelope {
    v: u8,
    nonce_hint: String,
    payload: String,
    aad: String,
}

#[derive(serde::Serialize)]
struct RevealRequest {
    manifest_signature: String,
    envelope: RevealEnvelope,
}

#[derive(serde::Deserialize)]
struct RevealResponse {
    nonce: String,
    ciphertext: String,
}

const NONCE_SIZE: usize = 12;

pub struct ApplyEngine {
    pub manifest: StepManifest,
    pub envelopes: Vec<DivineFixEnvelope>,
    pub key_token: String,
    pub manifest_signature: String,
    pub trace_id: String,
}

fn fetch_ephemeral_key(
    token: &str,
    config: &CliConfig,
) -> Result<GenericArray<u8, <Aes256Gcm as KeySizeUser>::KeySize>, Box<dyn Error>> {
    let client = Client::builder().timeout(config.http_timeout).build()?;
    let url = config.divine_key_url(token);
    let resp = client.get(url).send()?.error_for_status()?;
    let key = resp.json::<KeyResponse>()?;
    let bytes = general_purpose::STANDARD.decode(&key.base64_key)?;
    Ok(GenericArray::<u8, <Aes256Gcm as KeySizeUser>::KeySize>::clone_from_slice(&bytes))
}

impl ApplyEngine {
    #[must_use]
    pub const fn new(
        manifest: StepManifest,
        envelopes: Vec<DivineFixEnvelope>,
        key_token: String,
        manifest_signature: String,
        trace_id: String,
    ) -> Self {
        Self {
            manifest,
            envelopes,
            key_token,
            manifest_signature,
            trace_id,
        }
    }

    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn decrypt_step(&self, index: usize, config: &CliConfig) -> Result<String, String> {
        // 1) Verify manifest integrity
        let manifest_json =
            serde_json::to_string(&self.manifest).map_err(|e| format!("Hashing failed: {e}"))?;
        let digest = Sha256::digest(manifest_json.as_bytes());
        let computed_hash = format!("{digest:x}");
        if computed_hash != self.manifest_signature {
            return Err("Manifest integrity check failed.".to_string());
        }

        // 2) Pick envelope
        let envelope = self
            .envelopes
            .get(index)
            .ok_or_else(|| format!("No envelope at index {index}"))?;

        // Verify AAD locally
        let aad_input = format!("{}|{}|{}", self.key_token, self.manifest_signature, index);
        let aad_check = Sha256::digest(aad_input.as_bytes());
        let aad_b64 = general_purpose::STANDARD.encode(aad_check);
        if envelope.aad != aad_b64 {
            return Err("AAD verification failed".to_string());
        }

        // 3) Ask server to reveal this step to us (per-step rewrap)
        let client = Client::builder()
            .timeout(config.http_timeout)
            .build()
            .map_err(|e| format!("Client build error: {e}"))?;
        let url = config.reveal_step_url(&self.key_token);
        let req = RevealRequest {
            manifest_signature: self.manifest_signature.clone(),
            envelope: RevealEnvelope {
                v: envelope.v,
                nonce_hint: envelope.nonce_hint.clone(),
                payload: envelope.payload.clone(),
                aad: envelope.aad.clone(),
            },
        };
        let resp = client
            .post(url)
            .header("trace_id", &self.trace_id)
            .json(&req)
            .send()
            .map_err(|e| format!("Reveal POST failed: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("Reveal failed with status: {}", resp.status()));
        }
        let body = resp
            .json::<RevealResponse>()
            .map_err(|e| format!("Reveal response parse failed: {e}"))?;

        // 4) Decrypt with our ephemeral key
        let key = fetch_ephemeral_key(&self.key_token, config).map_err(|e| e.to_string())?;
        let cipher = Aes256Gcm::new(&key);

        let nonce_bytes = general_purpose::STANDARD
            .decode(&body.nonce)
            .map_err(|e| format!("nonce b64 decode error: {e}"))?;
        if nonce_bytes.len() != NONCE_SIZE {
            return Err(format!("Invalid nonce length: {}", nonce_bytes.len()));
        }
        let nonce = GenericArray::from_slice(&nonce_bytes);

        let ct = general_purpose::STANDARD
            .decode(&body.ciphertext)
            .map_err(|e| format!("ciphertext b64 decode error: {e}"))?;

        let decrypted = cipher
            .decrypt(nonce, ct.as_slice())
            .map_err(|e| format!("Decryption failed: {e}"))?;

        let redacted = if self.key_token.len() > 6 {
            format!("{}***", &self.key_token[..6])
        } else {
            "***".to_string()
        };
        info!(
            "Decrypted step {} with ephemeral key for token {}",
            index, redacted
        );

        String::from_utf8(decrypted).map_err(|e| format!("Invalid UTF-8: {e}"))
    }
}

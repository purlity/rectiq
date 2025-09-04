use crate::{
    config::CliConfig,
    divine::{
        apply::{ApplyEngine, DivineFixEnvelope},
        manifest::StepManifest,
    },
    remote::{
        ephemeral_key::fetch_ephemeral_key_from_server,
        request_builder::{compute_body_hash_b64, BuildMode, RequestBuilder},
    },
    security::aad::{verify_aad, FixTxError},
    symbolset::model::SymbolSet,
    utils::sort_json_key::sort_json_keys,
};
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use rectiq_crypto::gen_b64_token;
use rectiq_types::{EncodedPayload, ByteSpan};
use reqwest::{blocking::Client, header::CONTENT_TYPE};
use sha2::{Digest, Sha256};
use tracing::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

pub struct FixTransmitter {
    pub user_id: String,
    token: String,
    pub ephemeral_key: String,
    client: Client,
    endpoint: String,
    config: CliConfig,
}

#[derive(Deserialize)]
pub struct SealedParts {
    nonce: String,
    ciphertext: String,
}

#[derive(Deserialize)]
pub struct DivineFixBundleLocal {
    sealed: Vec<DivineFixEnvelope>,
    manifest: StepManifest,
    manifest_signature: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FixActionLite {
    pub span: ByteSpan,
    pub replacement: String,
}

impl FixTransmitter {
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn initialize(user_id: &str) -> Result<Self, String> {
        println!("🔧 FixTransmitter initializing!");

        let config = CliConfig::default();

        let token = env::var("RECTIQ_API_KEY").map_err(|_| "Missing RECTIQ_API_KEY".to_string())?;
        let endpoint = config.fix_url();

        let ephemeral_key = fetch_ephemeral_key_from_server(&token, &config)?;
        let client = Client::builder()
            .timeout(config.http_timeout)
            .build()
            .map_err(|e| format!("Client build error: {e}"))?;

        let token_preview = format!(
            "{}…{}",
            &token[..4.min(token.len())],
            &token[token.len().saturating_sub(4)..]
        );
        println!("🔑 Loaded token (masked): {token_preview}");
        println!(
            "🔐 Ephemeral key starts with: {}",
            &ephemeral_key[..8.min(ephemeral_key.len())]
        );

        Ok(Self {
            user_id: user_id.to_string(),
            token,
            ephemeral_key,
            client,
            endpoint,
            config,
        })
    }

    /// Send a FIX request using provided `SymbolSet` and already-encoded sketches.
    ///
    /// # Errors
    /// Returns [`FixTxError`] if network communication, hashing, AAD verification,
    /// or decryption/parsing of the server response fails.
    pub fn send_with(&self, symbolset: &SymbolSet, encoded: &str) -> Result<String, FixTxError> {
        println!("🔧 FixTransmitter sending (send_with)!");

        let trace_id = gen_b64_token(16);
        info!("trace_id={trace_id}");

        // Pre-compute hashes and wrapper
        let (key_bytes, key_hash) = self.compute_key_hash()?;
        let symbolset_hash = Self::compute_symbolset_hash(symbolset)?;
        let wrapper_json = self.build_wrapper_json(encoded)?;

        // AAD bits
        let (nonce, body_hash, ts) = Self::prepare_aad(&wrapper_json)?;

        // Build and send request
        let url = self.endpoint.clone();
        let req = self
            .build_request(&url, &wrapper_json, &nonce, &body_hash, &ts)?
            .header("X-Ephemeral-Key-Hash", key_hash)
            .header("X-SymbolSet-Hash", symbolset_hash)
            .header("trace_id", &trace_id);

        let response = req.send();
        match response {
            Ok(resp) => self.handle_response(resp, &key_bytes, &nonce, &body_hash, &ts, &trace_id),
            Err(e) => Err(FixTxError::Other(format!(
                "❌ Failed to send fix request: {e}"
            ))),
        }
    }

    fn compute_key_hash(&self) -> Result<(Vec<u8>, String), FixTxError> {
        let key_bytes = general_purpose::STANDARD
            .decode(&self.ephemeral_key)
            .map_err(|e| FixTxError::Other(format!("ephemeral key b64 decode error: {e}")))?;
        let key_hash = hex::encode(Sha256::digest(&key_bytes));
        Ok((key_bytes, key_hash))
    }

    fn compute_symbolset_hash(symbolset: &SymbolSet) -> Result<String, FixTxError> {
        let value = serde_json::to_value(symbolset)
            .map_err(|e| FixTxError::Other(format!("SymbolSet to_value error: {e}")))?;
        let sorted = sort_json_keys(value);
        let json = serde_json::to_string(&sorted)
            .map_err(|e| FixTxError::Other(format!("SymbolSet serialization error: {e}")))?;
        Ok(hex::encode(Sha256::digest(json.as_bytes())))
    }

    fn build_wrapper_json(&self, encoded: &str) -> Result<String, FixTxError> {
        let wrapper = EncodedPayload {
            payload: encoded.to_string(),
        };
        let wrapper_json = serde_json::to_string(&wrapper)
            .map_err(|e| FixTxError::Other(format!("Wrapper serialization error: {e}")))?;
        if self.config.is_dev() {
            eprintln!(">>> HTTP REQUEST BODY:\n{wrapper_json}");
        }
        Ok(wrapper_json)
    }

    fn prepare_aad(wrapper_json: &str) -> Result<(String, String, String), FixTxError> {
        let mut raw = [0u8; 16];
        getrandom::fill(&mut raw).map_err(|e| FixTxError::Other(format!("OS RNG failure: {e}")))?;
        let nonce = base64::engine::general_purpose::STANDARD.encode(raw);
        let body_hash = compute_body_hash_b64(wrapper_json.as_bytes());
        let ts = chrono::Utc::now().timestamp().to_string();
        Ok((nonce, body_hash, ts))
    }

    fn build_request(
        &self,
        url: &str,
        wrapper_json: &str,
        nonce: &str,
        body_hash: &str,
        ts: &str,
    ) -> Result<reqwest::blocking::RequestBuilder, FixTxError> {
        let rbuilder = RequestBuilder::new(&self.client);
        let (req, _aad_used) = rbuilder
            .post_json_with_aad(
                url,
                wrapper_json,
                Some(&self.user_id),
                &BuildMode::UseProvided {
                    nonce,
                    body_hash_b64: body_hash,
                    ts,
                },
            )
            .map_err(|e| FixTxError::Other(format!("build error: {e}")))?;
        Ok(req)
    }

    fn handle_response(
        &self,
        resp: reqwest::blocking::Response,
        key_bytes: &[u8],
        nonce: &str,
        body_hash: &str,
        ts: &str,
        trace_id: &str,
    ) -> Result<String, FixTxError> {
        let status = resp.status();
        if !status.is_success() {
            let body: String = resp.text().unwrap_or_default();
            return Err(FixTxError::Other(format!(
                "❌ Fix request failed [{status}]: {body}"
            )));
        }

        let headers = resp.headers().clone();
        let server_path = resp.url().path().to_owned();
        let aad_hdr = headers.get("X-Rectiq-AAD").and_then(|v| v.to_str().ok());
        verify_aad(aad_hdr, nonce, &server_path, body_hash, ts)?;

        let body: String = resp.text().unwrap_or_default();
        let ctype = headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("<none>");
        let preview: String = body.chars().take(160).collect();
        println!("📬 [CLI] Server responded with status: {status} | Content-Type: {ctype}");
        println!("⬇️  [CLI] Response preview: {preview}");

        if let Ok(parts) = serde_json::from_str::<SealedParts>(&body) {
            return self.decrypt_and_collect_actions(&parts, key_bytes, trace_id);
        }

        Ok(body)
    }

    fn decrypt_and_collect_actions(
        &self,
        parts: &SealedParts,
        key_bytes: &[u8],
        trace_id: &str,
    ) -> Result<String, FixTxError> {
        let key = aes_gcm::aead::generic_array::GenericArray::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        let nonce_bytes = general_purpose::STANDARD
            .decode(&parts.nonce)
            .map_err(|e| FixTxError::Other(format!("nonce b64 decode error: {e}")))?;
        if nonce_bytes.len() != 12 {
            return Err(FixTxError::Other(format!(
                "invalid nonce length: {}",
                nonce_bytes.len()
            )));
        }
        let nonce_ga = aes_gcm::aead::generic_array::GenericArray::from_slice(&nonce_bytes);

        let ct_bytes = general_purpose::STANDARD
            .decode(&parts.ciphertext)
            .map_err(|e| FixTxError::Other(format!("ciphertext b64 decode error: {e}")))?;

        let decrypted = cipher
            .decrypt(nonce_ga, ct_bytes.as_slice())
            .map_err(|e| FixTxError::Other(format!("outer decrypt failed: {e}")))?;
        let bundle_json = String::from_utf8(decrypted)
            .map_err(|e| FixTxError::Other(format!("outer decrypt invalid utf8: {e}")))?;

        let bundle: DivineFixBundleLocal = serde_json::from_str(&bundle_json).map_err(|e| {
            FixTxError::Other(format!(
                "Divine bundle parse failed: {e} | begins: {}",
                &bundle_json.chars().take(120).collect::<String>()
            ))
        })?;

        let engine = ApplyEngine::new(
            bundle.manifest.clone(),
            bundle.sealed.clone(),
            self.token.clone(),
            bundle.manifest_signature.clone(),
            trace_id.to_string(),
        );

        let mut actions: Vec<FixActionLite> = Vec::new();
        for idx in bundle.manifest.materialize_execution_order() {
            let step_json = engine
                .decrypt_step(idx, &self.config)
                .map_err(|e| FixTxError::Other(format!("reveal/decrypt step {idx} failed: {e}")))?;

            let lite: FixActionLite = serde_json::from_str(&step_json)
                .or_else(|_| {
                    #[derive(Deserialize)]
                    struct Envelope {
                        actions: Vec<FixActionLite>,
                    }
                    serde_json::from_str::<Envelope>(&step_json).map(|e| {
                        e.actions.into_iter().next().unwrap_or(FixActionLite {
                            span: ByteSpan { start: 0, end: 0 },
                            replacement: String::new(),
                        })
                    })
                })
                .map_err(|e| {
                    FixTxError::Other(format!(
                        "step parse failed: {e} | begins: {}",
                        &step_json.chars().take(120).collect::<String>()
                    ))
                })?;

            actions.push(lite);
        }

        let out = serde_json::to_string(&actions)
            .map_err(|e| FixTxError::Other(format!("actions serialization failed: {e}")))?;
        println!(
            "📦 [CLI] Collated {} actions from Divine bundle",
            actions.len()
        );
        Ok(out)
    }

    #[must_use]
    pub fn extract_key_base64(&self) -> String {
        println!("🔧 FixTransmitter extract_key_base64!");
        self.ephemeral_key.clone()
    }
}

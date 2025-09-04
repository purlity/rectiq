// rectiq-cli/src/remote/request_builder.rs

use sha2::{Digest, Sha256};
use base64::Engine;
use reqwest::{blocking::Client, header::CONTENT_TYPE};

#[derive(Debug, Clone)]
pub struct Aad {
    pub nonce: String,
    pub body_hash_b64: String,
    pub ts: String,
}

#[derive(Debug)]
pub enum BuildMode<'a> {
    /// Use caller-provided AAD (e.g., `FixRequestBuilder` already computed).
    UseProvided {
        nonce: &'a str,
        body_hash_b64: &'a str,
        ts: &'a str,
    },
    /// Compute AAD for the given JSON body here.
    Compute,
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("JSON body missing")]
    BodyMissing,
}

#[must_use]
#[inline]
pub fn compute_body_hash_b64(body: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(Sha256::digest(body))
}

pub struct RequestBuilder<'a> {
    client: &'a Client,
}

impl<'a> RequestBuilder<'a> {
    #[must_use]
    pub const fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Prepare a POST JSON request with Rectiq headers and AAD.
    /// Returns the prepared reqwest builder and the AAD used.
    ///
    /// # Panics
    ///
    /// Panics if OS RNG fails.
    ///
    /// # Errors
    ///
    /// Returns `BuildError::BodyMissing` if the JSON body is empty.
    pub fn post_json_with_aad(
        &self,
        url: &str,
        body_json: &str,
        auth_bearer: Option<&str>,
        mode: &BuildMode<'_>,
    ) -> Result<(reqwest::blocking::RequestBuilder, Aad), BuildError> {
        if body_json.is_empty() {
            return Err(BuildError::BodyMissing);
        }

        let (nonce, body_hash_b64, ts) = match *mode {
            BuildMode::UseProvided {
                nonce,
                body_hash_b64,
                ts,
            } => (nonce.to_string(), body_hash_b64.to_string(), ts.to_string()),
            BuildMode::Compute => {
                // 16 bytes, base64
                let mut raw = [0u8; 16];
                getrandom::fill(&mut raw).expect("OS RNG");
                let nonce_b64 = base64::engine::general_purpose::STANDARD.encode(raw);
                let body_hash_b64 = compute_body_hash_b64(body_json.as_bytes());
                let ts = chrono::Utc::now().timestamp().to_string();
                (nonce_b64, body_hash_b64, ts)
            }
        };

        let mut rb = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header("X-Rectiq-Nonce", &nonce)
            .header("X-Rectiq-Body-Hash", &body_hash_b64)
            .header("X-Rectiq-Timestamp", &ts)
            .body(body_json.to_string());

        if let Some(bearer) = auth_bearer {
            rb = rb.header("Authorization", format!("Bearer {bearer}"));
        }

        let aad = Aad {
            nonce,
            body_hash_b64,
            ts,
        };
        Ok((rb, aad))
    }
}

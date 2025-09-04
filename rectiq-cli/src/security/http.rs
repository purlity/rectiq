#![forbid(unsafe_code)]
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use super::aad::{sign_request, verify_response_aad};
use crate::keystore::default_store;
use zeroize::Zeroize;

pub struct HttpSigner {
    // future config fields
}

impl Default for HttpSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpSigner {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Build authorization and AAD headers for a request.
    ///
    /// # Errors
    /// Returns an error if the API key cannot be loaded from the keystore or if any header value is invalid UTF-8.
    pub fn prepare_headers(
        &self,
        api_key_name: &str,
        path: &str,
        body: &[u8],
    ) -> anyhow::Result<(HeaderMap, super::aad::SignedHeaders)> {
        let store = default_store();
        let mut token = store.get(api_key_name)?;
        let signed = sign_request(path, body);

        let mut hdrs = HeaderMap::new();
        hdrs.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_val = format!("Bearer {token}");
        hdrs.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_val).map_err(|e| anyhow::anyhow!(e))?,
        );
        hdrs.insert("X-Rectiq-Nonce", HeaderValue::from_str(&signed.nonce)?);
        hdrs.insert(
            "X-Rectiq-Body-Hash",
            HeaderValue::from_str(&signed.body_hash_b64)?,
        );
        hdrs.insert(
            "X-Rectiq-Timestamp",
            HeaderValue::from_str(&signed.timestamp_rfc3339)?,
        );
        hdrs.insert("X-Rectiq-AAD", HeaderValue::from_str(&signed.aad)?);

        drop(auth_val);
        token.zeroize();
        Ok((hdrs, signed))
    }

    /// Verify the server’s echoed AAD header against what was sent.
    ///
    /// # Errors
    /// Returns an error if the response lacks the expected header or if verification fails.
    pub fn verify_response(
        &self,
        sent: &super::aad::SignedHeaders,
        headers: &HeaderMap,
    ) -> anyhow::Result<()> {
        let echoed = headers
            .get("X-Rectiq-Resp-AAD")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| anyhow::anyhow!("missing response AAD"))?;
        verify_response_aad(sent, echoed, 300).map_err(|e| anyhow::anyhow!(e))
    }
}

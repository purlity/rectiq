// rectiq-cli/src/remote/ephemeral_key.rs

use crate::{
    config::CliConfig,
    remote::request_builder::{BuildMode, RequestBuilder},
    security::aad::verify_aad,
};
use reqwest::blocking::Client;

#[derive(serde::Deserialize)]
struct KeyResponse {
    base64_key: String,
}

/// # Errors
/// Returns an error when the input payload fails validation.
pub fn fetch_ephemeral_key_from_server(token: &str, config: &CliConfig) -> Result<String, String> {
    println!("🔑 [CLI] Fetching ephemeral key for token: '{token}'");

    let client = Client::builder()
        .timeout(config.http_timeout)
        .build()
        .map_err(|e| format!("Client build error: {e}"))?;

    let url = config.divine_key_url(token);
    let rb = RequestBuilder::new(&client);
    let (req, aad) = rb
        .post_json_with_aad(&url, "{}", Some(token), &BuildMode::Compute)
        .map_err(|e| format!("request build failed: {e}"))?;

    let response = req
        .send()
        .map_err(|e| format!("Failed to reach divine-key API: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Divine key server returned error: {}",
            response.status()
        ));
    }

    let headers = response.headers().clone();
    let server_path = response.url().path().to_owned();
    let aad_hdr = headers.get("X-Rectiq-AAD").and_then(|v| v.to_str().ok());
    verify_aad(
        aad_hdr,
        &aad.nonce,
        &server_path,
        &aad.body_hash_b64,
        &aad.ts,
    )
    .map_err(|e| format!("AAD verify failed: {e}"))?;

    let key_response: KeyResponse = response
        .json()
        .map_err(|e| format!("Failed to parse divine-key response: {e}"))?;

    let redacted = if key_response.base64_key.len() > 6 {
        format!("{}***", &key_response.base64_key[..6])
    } else {
        "***".to_string()
    };
    println!("✅ [CLI] Received base64 key: {redacted}");
    Ok(key_response.base64_key)
}

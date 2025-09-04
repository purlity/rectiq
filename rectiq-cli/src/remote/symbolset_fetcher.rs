// rectiq-cli/src/remote/symbolset_fetcher.rs

use crate::{
    config::CliConfig,
    remote::{
        helper::{collect_chars_from_sketch, insert_chars},
        request_builder::{BuildMode, RequestBuilder},
    },
    security::aad::verify_aad,
    symbolset::{dto::SymbolSetMetadata, model::SymbolSet},
};
use rectiq_types::SketchNode;
use reqwest::blocking::Client;
use serde_json;
use std::{
    collections::{HashMap, HashSet},
};

/// # Errors
/// Returns an error when the input payload fails validation.
pub fn fetch_symbolset_from_server(
    user_id: &str,
    sketches: &[SketchNode],
) -> Result<(SymbolSet, SymbolSetMetadata), String> {
    println!("📦 [CLI] Requesting SymbolSet for user_id = '{user_id}'");
    let mut required_set = HashSet::new();

    for sketch in sketches {
        collect_chars_from_sketch(sketch, &mut required_set);
    }

    // Include wrapper key to capture characters like 'c' in "sketches"
    insert_chars("sketches", &mut required_set);

    // Ensure essential JSON syntax characters are always included
    let json_syntax = ['{', '}', '[', ']', ':', ','];
    for c in json_syntax {
        required_set.insert(c);
    }
    // Always-required chars that must be included for masking, even if sketches omit them
    let always_required = ['\\', '_'];
    for c in always_required {
        required_set.insert(c);
    }
    let required_chars: Vec<char> = required_set.into_iter().collect();

    let cfg = CliConfig::from_env_or_infer();
    let client = Client::builder()
        .timeout(cfg.http_timeout)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let payload = serde_json::json!({
        "user_id": user_id,
        "required_chars": required_chars
    });

    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| format!("symbolset payload serialize failed: {e}"))?;

    let url = cfg.symbolset_init_url();

    println!(
        "📤 Required chars being sent ({}): {:?}",
        required_chars.len(),
        required_chars
    );

    let rb = RequestBuilder::new(&client);
    let (req, aad) = rb
        .post_json_with_aad(&url, &payload_json, Some(user_id), &BuildMode::Compute)
        .map_err(|e| format!("request build failed: {e}"))?;

    let response = req
        .send()
        .map_err(|e| format!("SymbolSet request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "❌ SymbolSet server returned error: {}",
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

    let meta: SymbolSetMetadata = response
        .json()
        .map_err(|e| format!("❌ Failed to deserialize SymbolSetMetadata: {e}"))?;

    // Preserve the deterministic order used when the server generated `masked`
    let mut required_sorted = meta.required.clone();
    required_sorted.sort_unstable();

    let symbol_map: HashMap<char, String> = required_sorted
        .into_iter()
        .zip(meta.masked.iter().map(ToString::to_string))
        .collect();

    println!(
        "📦 [CLI] Received SymbolSet map ({} chars):",
        meta.required.len()
    );

    let symbolset = SymbolSet::from_map(symbol_map);
    Ok((symbolset, meta))
}

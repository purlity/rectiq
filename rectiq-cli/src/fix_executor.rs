// rectiq-cli/src/fix_executor.rs
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use base64::{Engine, engine::general_purpose};
use serde::Deserialize;
use serde_json;
use rectiq_types::ByteSpan;

#[derive(Deserialize, Debug)]
struct RawFixStep {
    start: usize,
    end: usize,
    replace: String,
}

#[derive(Deserialize, Debug)]
struct PlanActionMinimal {
    span: ByteSpan,
    #[serde(default)]
    replacement: String,
}

#[derive(Deserialize, Debug)]
struct FixActionLite {
    span: ByteSpan,
    #[serde(default)]
    replacement: String,
}

#[derive(Deserialize, Debug)]
struct FixPlanEnvelope {
    actions: Vec<PlanActionMinimal>,
}

#[derive(Deserialize, Debug)]
struct SealedParts {
    nonce: String,
    ciphertext: String,
}

pub struct FixExecutor;

impl FixExecutor {
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn apply_from_blob(
        input: &str,
        encrypted_blob: &str,
        base64_key: &str,
    ) -> Result<String, String> {
        println!("🔧 FixExecutor triggered!");

        // 0) FAST-PATH: caller already provided plaintext FixAction JSON
        if let Some(steps) = parse_any_to_steps(encrypted_blob) {
            println!("🛣️  Fast-path: plaintext FixAction payload (no decrypt)");
            return apply_steps(input, steps);
        }

        // 1) Decide which cipher representation we received (no Divine bundle here; transmitter handles it)
        let (cipher_b64, origin_hint) =
            if let Ok(parts) = serde_json::from_str::<SealedParts>(encrypted_blob) {
                // Server returned explicit JSON parts: { nonce: b64(12), ciphertext: b64(ct||tag) }
                println!("📦 Received SealedParts with explicit nonce+ciphertext");
                let nonce_bytes = general_purpose::STANDARD
                    .decode(&parts.nonce)
                    .map_err(|e| format!("nonce b64 decode error: {e}"))?;
                if nonce_bytes.len() != 12 {
                    return Err(format!("Invalid nonce length: {}", nonce_bytes.len()));
                }
                let ct_bytes = general_purpose::STANDARD
                    .decode(&parts.ciphertext)
                    .map_err(|e| format!("ciphertext b64 decode error: {e}"))?;

                // Build a single base64 blob in layout #1 expected by the decryptor: [nonce(12)] || [ciphertext||tag]
                let mut combined = Vec::with_capacity(12 + ct_bytes.len());
                combined.extend_from_slice(&nonce_bytes);
                combined.extend_from_slice(&ct_bytes);
                let joined_b64 = general_purpose::STANDARD.encode(&combined);
                (joined_b64, "parts.nonce+ciphertext")
            } else if let Ok(s) = serde_json::from_str::<String>(encrypted_blob) {
                // Server returned a JSON string containing base64
                println!("📦 Received JSON string ciphertext (len={})", s.len());
                (s, "json-string")
            } else {
                // Treat raw body as base64
                (encrypted_blob.to_string(), "raw-base64")
            };
        println!(
            "🔐 Cipher origin: {} (b64-len={})",
            origin_hint,
            cipher_b64.len()
        );

        // 2) Decrypt
        let decrypted = match decrypt_blob_with_key(&cipher_b64, base64_key) {
            Ok(s) => s,
            Err(e) => {
                // Print short diagnostics to help align server/cli crypto framing
                let b = base64::engine::general_purpose::STANDARD
                    .decode(&cipher_b64)
                    .map_err(|e2| format!("{e} | and base64 decode failed: {e2}"))?;
                let preview: Vec<u8> = b.iter().copied().take(16).collect();
                return Err(format!(
                    "{} | decoded-bytes={} first16={:02X?}",
                    e,
                    b.len(),
                    preview
                ));
            }
        };

        // 3) Parse and apply decrypted steps (supports multiple shapes)
        if let Some(steps) = parse_any_to_steps(&decrypted) {
            return apply_steps(input, steps);
        }
        Err(format!(
            "FixPlan parse failed: unsupported shape. Decrypted payload begins with: {}",
            &decrypted.chars().take(120).collect::<String>()
        ))
    }
}

fn decrypt_blob_with_key(ciphertext: &str, base64_key: &str) -> Result<String, String> {
    let key_bytes = general_purpose::STANDARD
        .decode(base64_key)
        .map_err(|e| e.to_string())?;
    let key = GenericArray::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let data = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| format!("Base64 decode error: {e}"))?;

    if data.len() < 12 + 16 {
        return Err(format!(
            "❌ Encrypted blob too short ({} bytes). Need at least nonce(12)+tag(16).",
            data.len()
        ));
    }

    // Helper to finish by attempting UTF-8 conversion
    let finish = |bytes: Vec<u8>| -> Result<String, String> {
        String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {e}"))
    };

    // Try 1: [nonce(12)] + [ciphertext || tag]
    {
        let (nonce_bytes, rest) = data.split_at(12);
        let nonce = GenericArray::from_slice(nonce_bytes);
        if let Ok(plain) = cipher.decrypt(nonce, rest) {
            return finish(plain);
        }
    }

    // Try 2: [nonce(12)] + [tag(16)] + [ciphertext]
    {
        let (nonce_bytes, rest) = data.split_at(12);
        if rest.len() > 16 {
            let (tag, ct) = rest.split_at(16);
            let mut ct_then_tag = Vec::with_capacity(ct.len() + tag.len());
            ct_then_tag.extend_from_slice(ct);
            ct_then_tag.extend_from_slice(tag);
            let nonce = GenericArray::from_slice(nonce_bytes);
            if let Ok(plain) = cipher.decrypt(nonce, ct_then_tag.as_slice()) {
                return finish(plain);
            }
        }
    }

    // Try 3: [ciphertext || tag] + [nonce(12)]
    {
        let (body, nonce_bytes) = data.split_at(data.len() - 12);
        let nonce = GenericArray::from_slice(nonce_bytes);
        if let Ok(plain) = cipher.decrypt(nonce, body) {
            return finish(plain);
        }
    }

    // Try 4: [tag(16)] + [ciphertext] + [nonce(12)]
    {
        if data.len() > (16 + 12) {
            let (tag, rest) = data.split_at(16);
            let (ct, nonce_bytes) = rest.split_at(rest.len() - 12);
            let mut ct_then_tag = Vec::with_capacity(ct.len() + tag.len());
            ct_then_tag.extend_from_slice(ct);
            ct_then_tag.extend_from_slice(tag);
            let nonce = GenericArray::from_slice(nonce_bytes);
            if let Ok(plain) = cipher.decrypt(nonce, ct_then_tag.as_slice()) {
                return finish(plain);
            }
        }
    }

    Err(format!(
        "Decryption failed for all layouts | total-bytes={} first16={:02X?} last16={:02X?}",
        data.len(),
        &data.iter().copied().take(16).collect::<Vec<u8>>(),
        &data
            .iter()
            .copied()
            .rev()
            .take(16)
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .collect::<Vec<u8>>()
    ))
}

fn parse_any_to_steps(blob: &str) -> Option<Vec<RawFixStep>> {
    // Try direct array of RawFixStep
    if let Ok(steps) = serde_json::from_str::<Vec<RawFixStep>>(blob) {
        return Some(steps);
    }
    // Try an object with `actions: [{ span: (s,e), replacement: "" }]`
    if let Ok(env) = serde_json::from_str::<FixPlanEnvelope>(blob) {
        return Some(
            env.actions
                .into_iter()
                .map(|a| RawFixStep {
                    start: a.span.start,
                    end: a.span.end,
                    replace: a.replacement,
                })
                .collect(),
        );
    }
    // Try a vec of minimal actions
    if let Ok(actions) = serde_json::from_str::<Vec<PlanActionMinimal>>(blob) {
        return Some(
            actions
                .into_iter()
                .map(|a| RawFixStep {
                    start: a.span.start,
                    end: a.span.end,
                    replace: a.replacement,
                })
                .collect(),
        );
    }
    // Try single action
    if let Ok(single) = serde_json::from_str::<FixActionLite>(blob) {
        return Some(vec![RawFixStep {
            start: single.span.start,
            end: single.span.end,
            replace: single.replacement,
        }]);
    }
    None
}

fn apply_steps(input: &str, mut steps: Vec<RawFixStep>) -> Result<String, String> {
    // Apply from the back to keep spans stable
    steps.sort_by_key(|s| s.start);
    let mut output = input.to_string();
    for step in steps.into_iter().rev() {
        if step.start >= step.end || step.end > output.len() {
            return Err("Invalid fix range".into());
        }
        #[cfg(debug_assertions)]
        eprintln!("🩹 Fix: {}..{} = {:?}", step.start, step.end, step.replace);
        output.replace_range(step.start..step.end, &step.replace);
    }
    Ok(output)
}

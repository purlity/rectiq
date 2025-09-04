// rectiq-cli/src/utils/crypto.rs
use crate::{commands::init_symbolset, symbolset::manager::SymbolSetManager};
use rectiq_types::{FixHint, FixHintValue as V, JsonPath, JsonPathSegment, SuggestedValueHint as S};
use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, KeyInit, generic_array::GenericArray},
};
use base64::{Engine as _, engine::general_purpose};
use getrandom::{u64};
use once_cell::sync::OnceCell;
use typenum::U32;
use std::{borrow::Cow, sync::Once};

static MASK_INIT: Once = Once::new();

static KEY_CELL: OnceCell<GenericArray<u8, U32>> = OnceCell::new();

/// # Errors
/// Returns an error when the input payload fails validation.
pub fn decrypt_blob(ciphertext: &str) -> Result<String, String> {
    let key = resolve_decryption_key()?;
    let cipher = Aes256Gcm::new(key);

    // Decode the base64-encoded ciphertext
    let data = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| format!("❌ Failed to decode base64 ciphertext: {e}"))?;

    // Ensure the data is long enough to contain a nonce
    if data.len() < 12 {
        return Err("❌ Encrypted blob too short to contain a valid nonce.".into());
    }

    // Split the nonce and the encrypted bytes
    let (nonce_bytes, encrypted_bytes) = data.split_at(12);
    let nonce = GenericArray::from_slice(nonce_bytes);

    // Attempt decryption
    let decrypted = cipher
        .decrypt(nonce, encrypted_bytes)
        .map_err(|e| format!("❌ Decryption failed: {e}"))?;

    // Convert decrypted bytes to UTF-8 string
    String::from_utf8(decrypted).map_err(|e| format!("❌ Invalid UTF-8 in decrypted output: {e}"))
}

fn resolve_decryption_key() -> Result<&'static GenericArray<u8, U32>, String> {
    // Initialize the key lazily and store it in a global OnceCell
    KEY_CELL.get_or_try_init(|| {
        let symbolset = SymbolSetManager::load_from_default()
            .map_err(|e| format!("❌ SymbolSet not found: {e}"))?;

        let key_bytes = general_purpose::STANDARD
            .decode(&symbolset.base64_key)
            .map_err(|e| format!("❌ Failed to decode base64 key: {e}"))?;

        if key_bytes.len() != 32 {
            return Err(format!(
                "❌ Invalid key length: expected 32 bytes, got {}.",
                key_bytes.len()
            ));
        }

        // SAFETY: OnceCell ensures the data is stored exactly once and lives for the
        // program's lifetime, giving us a true `'static` reference without leaking.
        Ok(GenericArray::clone_from_slice(&key_bytes))
    })
}

// Helper promoted to module scope for clarity and lint compliance.
const fn is_json_delimiter(c: char) -> bool {
    matches!(c, '{' | '}' | '[' | ']' | ':' | ',' | '\"')
}

/// Masks an input string, preserving JSON structure delimiters and character count,
/// using replacement patterns from the `SymbolSet`.
///
/// # Panics
/// Panics if the `SymbolSet` cannot be loaded from disk.
pub fn mask(input: &str) -> Cow<'_, str> {
    let symbolset_path = SymbolSetManager::default_path();
    MASK_INIT.call_once(|| {
        if !symbolset_path.exists() {
            println!("✅ Redaction layer activated. You're now protected.");
            init_symbolset(); // triggers creation
        }
    });
    // Load SymbolSet (must exist after init)
    let symbolset = SymbolSetManager::load_from_default().expect("SymbolSet missing or unreadable");
    let redacted_patterns = &symbolset.replacement_patterns;
    let default_pattern = symbolset.redacted_char;
    let masked: String = input
        .chars()
        .map(|c| {
            if is_json_delimiter(c) {
                c
            } else {
                // Pick a masking char with same width as input char
                if redacted_patterns.is_empty() {
                    default_pattern
                } else {
                    // OS CSPRNG: single 64-bit draw for index selection
                    let rnd = u64().expect("OS RNG failure");
                    let len_u64 =
                        u64::try_from(redacted_patterns.len()).expect("pattern len fits in u64");
                    let idx = usize::try_from(rnd % len_u64).expect("index fits in usize");
                    redacted_patterns[idx]
                        .chars()
                        .next()
                        .unwrap_or(default_pattern)
                }
            }
        })
        .collect();
    Cow::Owned(masked)
}

#[must_use]
/// Mask a JSON Pointer-like path while preserving separators such as `#` and `/`.
/// Each segment between slashes is masked using the SymbolSet-based masker.
pub fn mask_pointer_str(s: &str) -> String {
    // Handle optional leading '#'
    let (prefix, rest) = s
        .strip_prefix('#')
        .map_or(("", s), |stripped| ("#", stripped));
    // Split on '/', mask each segment, and rejoin with '/'
    let masked: Vec<String> = rest.split('/').map(|seg| mask(seg).into_owned()).collect();
    format!("{prefix}{}", masked.join("/"))
}

#[must_use]
/// Mask a structured JSON path (segments) while preserving its structure.
pub fn mask_json_path(path: &JsonPath<'_>) -> JsonPath<'static> {
    JsonPath(
        path.0
            .iter()
            .map(|seg: &JsonPathSegment<'_>| match seg {
                JsonPathSegment::Str(s) => {
                    // Mask only the key text; preserve structure
                    JsonPathSegment::Str(Cow::Owned(mask(s.as_ref()).into_owned()))
                }
                JsonPathSegment::Index(i) => JsonPathSegment::Index(*i),
            })
            .collect(),
    )
}

#[must_use]
pub fn mask_fix_hint(mut h: FixHint<'static>) -> FixHint<'static> {
    h.fields = h
        .fields
        .into_iter()
        .map(|(k, v)| {
            let v = match v {
                V::Str(s) => V::Str(Cow::Owned(mask(&s).into_owned())),
                V::StrList(list) => V::StrList(
                    list.into_iter()
                        .map(|s| Cow::Owned(mask(&s).into_owned()))
                        .collect(),
                ),
                V::Suggestion(S::StaticStr(s)) => {
                    V::Suggestion(S::StaticStr(Cow::Owned(mask(&s).into_owned())))
                }
                V::Suggestion(S::ReplaceWithEscape(s)) => {
                    V::Suggestion(S::ReplaceWithEscape(Cow::Owned(mask(&s).into_owned())))
                }
                other => other, // numbers, bools, paths, etc are safe
            };
            (k, v)
        })
        .collect();
    h
}

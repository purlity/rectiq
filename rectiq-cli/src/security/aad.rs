#![forbid(unsafe_code)]
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;
use subtle::ConstantTimeEq;

pub struct SignedHeaders {
    pub nonce: String,
    pub body_hash_b64: String,
    pub timestamp_rfc3339: String,
    /// Canonical AAD string: `nonce|path|body_hash|ts`
    pub aad: String,
}

/// Sign a request by producing the canonical AAD and signed headers.
///
/// # Panics
/// Panics if the system time cannot be formatted as RFC3339.
#[must_use]
pub fn sign_request(path: &str, body: &[u8]) -> SignedHeaders {
    let nonce = Uuid::new_v4().to_string();
    let bh = Sha256::digest(body);
    let body_hash_b64 = STANDARD_NO_PAD.encode(bh);
    let ts = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
    let aad = format!("{nonce}|{path}|{body_hash_b64}|{ts}");
    SignedHeaders {
        nonce,
        body_hash_b64,
        timestamp_rfc3339: ts,
        aad,
    }
}

/// Verify the server’s echoed AAD header strictly matches what we sent.
/// `resp_aad` is the raw header value (e.g., "nonce|/api/fix|...|ts").
///
/// # Errors
/// Returns an error if the echoed AAD does not match the signed headers.
pub fn verify_response_aad(
    sent: &SignedHeaders,
    resp_aad: &str,
    max_skew_secs: i64,
) -> Result<(), &'static str> {
    if resp_aad != sent.aad {
        return Err("aad_mismatch");
    }
    let _ = max_skew_secs;
    Ok(())
}

// -----------------------------------------------------------------------------
// Legacy helpers used by existing code paths.
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub enum FixTxError {
    AadMissing,
    AadMismatch,
    Other(String),
}

impl std::fmt::Display for FixTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AadMissing => write!(f, "AAD missing"),
            Self::AadMismatch => write!(f, "AAD mismatch"),
            Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for FixTxError {}

/// Verify an AAD header against expected components.
///
/// # Errors
/// Returns `FixTxError::AadMissing` if no header is present, or
/// `FixTxError::AadMismatch` if the header does not match.
pub fn verify_aad(
    aad_header: Option<&str>,
    nonce: &str,
    path: &str,
    body_hash: &str,
    ts: &str,
) -> Result<(), FixTxError> {
    let expected = format!("{nonce}|{path}|{body_hash}|{ts}");
    let header = aad_header.ok_or(FixTxError::AadMissing)?;
    let expected_hash = Sha256::digest(expected.as_bytes());
    let header_hash = Sha256::digest(header.as_bytes());
    if expected_hash
        .as_slice()
        .ct_eq(header_hash.as_slice())
        .into()
    {
        Ok(())
    } else {
        Err(FixTxError::AadMismatch)
    }
}

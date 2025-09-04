#![forbid(unsafe_code)]
use rectiq_cli::security::aad::{sign_request, verify_response_aad};

#[test]
fn aad_roundtrip() {
    let body = br#"{"x":1}"#;
    let s = sign_request("/api/fix", body);
    assert!(verify_response_aad(&s, &s.aad, 300).is_ok());
}

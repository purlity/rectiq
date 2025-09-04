use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use reqwest::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use rectiq_cli::{
    config::CliConfig,
    remote::request_builder::{compute_body_hash_b64, BuildMode, RequestBuilder},
    security::aad::{verify_aad, FixTxError},
};
use reqwest::blocking::Client;

#[test]
fn fix_response_aad_headers_are_computed() {
    // Arrange: known body and key so hashes are predictable.
    let body = "{\"sketches\":[1,2,3]}".to_string();

    let client = Client::new();
    let cfg = CliConfig::default();

    let (rb, _aad) = RequestBuilder::new(&client)
        .post_json_with_aad(&cfg.fix_url(), &body, Some("user_123"), &BuildMode::Compute)
        .expect("builder ok");

    let req = rb.build().expect("request build ok");

    // Assert headers presence (case-insensitive)
    // Authorization
    let auth = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    assert_eq!(auth, Some("Bearer user_123"));

    // Content type
    let ct = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());
    assert_eq!(ct, Some("application/json"));

    // Body hash (b64 of sha256(body))
    let expected_body_b64 = compute_body_hash_b64(body.as_bytes());
    let body_hash = req
        .headers()
        .get(HeaderName::from_static("x-rectiq-body-hash"))
        .and_then(|v| v.to_str().ok());
    assert_eq!(body_hash, Some(expected_body_b64.as_str()));

    // Nonce is base64 and decodes to 16 bytes
    let nonce_b64 = req
        .headers()
        .get(HeaderName::from_static("x-rectiq-nonce"))
        .and_then(|v| v.to_str().ok())
        .expect("nonce present");
    let decoded = B64.decode(nonce_b64).expect("nonce decodes");
    assert_eq!(decoded.len(), 16, "nonce must be 16 bytes");

    // Timestamp is numeric seconds
    let ts = req
        .headers()
        .get(HeaderName::from_static("x-rectiq-timestamp"))
        .and_then(|v| v.to_str().ok())
        .expect("timestamp present");
    assert!(ts.parse::<u64>().is_ok(), "timestamp must be numeric");
}

fn compute_aad(nonce: &str, path: &str, body_hash: &str, ts: &str) -> String {
    format!("{nonce}|{path}|{body_hash}|{ts}")
}

#[test]
fn compute_aad_is_stable() {
    let nonce = "n123";
    let path = "/api/fix";
    let body_hash = "h456";
    let ts = "789";
    let aad = compute_aad(nonce, path, body_hash, ts);
    assert_eq!(aad, "n123|/api/fix|h456|789");
}

#[test]
fn verify_aad_happy_path() {
    let nonce = "n123";
    let path = "/api/fix";
    let body_hash = "h456";
    let ts = "789";
    let aad = compute_aad(nonce, path, body_hash, ts);
    assert!(verify_aad(Some(&aad), nonce, path, body_hash, ts).is_ok());
}

#[test]
fn verify_aad_mismatched_path_errors() {
    let nonce = "n123";
    let path = "/api/fix";
    let body_hash = "h456";
    let ts = "789";
    let aad = compute_aad(nonce, path, body_hash, ts);
    let err = verify_aad(Some(&aad), nonce, "/different", body_hash, ts)
        .expect_err("expected AadMismatch");
    assert!(matches!(err, FixTxError::AadMismatch));
}

#[test]
fn verify_aad_missing_header_errors() {
    let nonce = "n123";
    let path = "/api/fix";
    let body_hash = "h456";
    let ts = "789";
    let err = verify_aad(None, nonce, path, body_hash, ts).expect_err("expected AadMissing");
    assert!(matches!(err, FixTxError::AadMissing));
}

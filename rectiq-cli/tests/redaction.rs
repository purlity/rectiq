#![forbid(unsafe_code)]
use rectiq_cli::security::redact::redact_headers;

#[test]
fn headers_are_redacted() {
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("Authorization", "Bearer abc".parse().unwrap());
    h.insert("X-Admin-Key", "sekret".parse().unwrap());
    let red = redact_headers(h);
    assert_eq!(red.get("Authorization").unwrap(), "Bearer [REDACTED]");
    assert_eq!(red.get("X-Admin-Key").unwrap(), "[REDACTED]");
}

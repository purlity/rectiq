use rectiq_cli::{
    config::CliConfig,
    remote::request_builder::{BuildMode, RequestBuilder},
};
use reqwest::blocking::Client;
use std::collections::HashSet;

#[test]
fn nonces_are_unique_across_builds() {
    let body = "{\"x\":1}".to_string();
    let client = Client::new();
    let cfg = CliConfig::default();
    let mut set = HashSet::new();
    for _ in 0..64 {
        let (_rb, aad) = RequestBuilder::new(&client)
            .post_json_with_aad(&cfg.fix_url(), &body, Some("u"), &BuildMode::Compute)
            .expect("build ok");
        assert!(set.insert(aad.nonce), "nonce must be unique per request");
    }
    assert_eq!(set.len(), 64, "expected 64 unique nonces");
}

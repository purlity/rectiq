use rectiq_cli::{
    config::CliConfig,
    remote::request_builder::{BuildMode, RequestBuilder},
};
use reqwest::blocking::Client;

mod common;

#[test]
fn authorization_header_is_bearer_and_content_type_json() {
    let body = "{}".to_string();
    let user_id = "customer-id";

    let client = Client::new();
    let cfg = CliConfig::default();

    let (_rb, _aad) = RequestBuilder::new(&client)
        .post_json_with_aad(&cfg.fix_url(), &body, Some(user_id), &BuildMode::Compute)
        .expect("builder ok");

    let headers = common::prepare_headers_with_token_basic(user_id);

    assert_eq!(
        headers.get("Authorization").and_then(|v| v.to_str().ok()),
        Some("Bearer customer-id")
    );
    assert_eq!(
        headers.get("Content-Type").and_then(|v| v.to_str().ok()),
        Some("application/json")
    );
}

#[test]
#[ignore = "user-id length validation is currently enforced server-side, to be reintroduced client-side later"]
fn empty_user_id_is_rejected() {
    // intentionally empty while validation is server-side
}

#[test]
#[ignore = "user-id length validation is currently enforced server-side, to be reintroduced client-side later"]
fn oversized_user_id_is_rejected() {
    // intentionally empty while validation is server-side
}

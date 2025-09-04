#![forbid(unsafe_code)]
use rectiq_cli::security::http::HttpSigner;

#[ignore = "requires live API server"]
#[tokio::test]
async fn nonce_reuse_hits_409() {
    let api = std::env::var("RECTIQ_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let key_name = std::env::var("RECTIQ_KEY_NAME").unwrap_or_else(|_| "default".into());
    let body = serde_json::json!({"payload":"test"});
    let body_bytes = serde_json::to_vec(&body).unwrap();
    let signer = HttpSigner::new();
    let path = "/api/fix";
    let (hdrs, sent) = signer
        .prepare_headers(&key_name, path, &body_bytes)
        .unwrap();

    let client = reqwest::Client::new();
    let url = format!("{api}{path}");
    let r1 = client
        .post(&url)
        .headers(hdrs.clone())
        .body(body_bytes.clone())
        .send()
        .await
        .unwrap();
    let _ = signer.verify_response(&sent, r1.headers());

    let r2 = client
        .post(&url)
        .headers(hdrs)
        .body(body_bytes)
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 409);
}

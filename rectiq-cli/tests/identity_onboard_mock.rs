use axum::{
    routing::{get, post},
    Json, Router,
    http::{HeaderMap, StatusCode},
};
use rectiq_cli::{config::CliConfig, identity::IdentityClient};
use serde_json::{json, Value};
use std::thread;
use std::sync::mpsc;
use assert_fs::prelude::*;

fn spawn_mock() -> String {
    let app = Router::new()
        .route(
            "/api/v1/identity/device-start",
            post(|Json(_): Json<Value>| async move {
                let body = json!({
                    "verification_uri": "https://example.com/verify",
                    "user_code": "ABCD-1234",
                    "device_code": "code-xyz",
                    "interval": 1,
                });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            "/api/v1/identity/device-complete",
            post(|Json(_): Json<Value>| async move {
                let body = json!({ "refresh_token": "r_tok" });
                (StatusCode::OK, Json(body))
            }),
        )
        .route(
            "/api/v1/auth/token",
            post(|| async { (StatusCode::OK, Json(json!({ "access_token": "a_tok" }))) }),
        )
        .route(
            "/api/v1/whoami",
            get(|headers: HeaderMap| async move {
                let ok = headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .contains("a_tok");
                if ok {
                    (
                        StatusCode::OK,
                        Json(json!({ "org": "acme", "user": "tester", "plan": "pro" })),
                    )
                } else {
                    (StatusCode::UNAUTHORIZED, Json(json!({})))
                }
            }),
        )
        .route(
            "/api/v1/devices/register",
            post(|| async { (StatusCode::OK, Json(json!({ "device_id": "dev-1" }))) }),
        )
        .route(
            "/api/v1/keys",
            post(|| async { (StatusCode::OK, Json(json!({ "secret_full": "api_abc" }))) }),
        )
        .route(
            "/api/v1/ping",
            get(|headers: HeaderMap| async move {
                if headers.get("dpop").is_some() {
                    (StatusCode::OK, Json(json!({})))
                } else {
                    (StatusCode::BAD_REQUEST, Json(json!({})))
                }
            }),
        );
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tx.send(addr.port()).unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    });
    let port = rx.recv().unwrap();
    format!("http://127.0.0.1:{port}/api")
}

#[test]
fn onboard_and_ping_flow() {
    // Setup env
    let base = spawn_mock();
    unsafe { std::env::set_var("RECTIQ_ENV", "production") };
    unsafe { std::env::set_var("RECTIQ_API_BASE", &base) };
    unsafe { std::env::set_var("RECTIQ_INSECURE_ALLOW_FILE", "1") };
    let tmp = assert_fs::TempDir::new().unwrap();
    let secrets_path = tmp.child("secrets.json");
    unsafe { std::env::set_var("RECTIQ_INSECURE_FILE", secrets_path.path()) };

    let cfg = CliConfig::default();
    let id = IdentityClient::new(cfg).unwrap();
    id.onboard("user@example.com").expect("onboard ok");

    // Verify secrets persisted
    let data = std::fs::read_to_string(secrets_path.path()).expect("secrets file");
    assert!(data.contains("rectiq:default:acme/tester:refresh"));
    assert!(data.contains("rectiq:default:acme/tester:api_key"));
    assert!(data.contains("rectiq:default:acme/tester:device"));

    id.ping("acme/tester").expect("ping ok");
}

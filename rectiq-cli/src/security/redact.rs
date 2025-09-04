#![forbid(unsafe_code)]
use std::panic;
use tracing::Level;

const REDACT_KEYS: &[&str] = &[
    "Authorization",
    "X-Admin-Key",
    "X-Rectiq-Nonce",
    "X-Rectiq-Body-Hash",
    "X-Rectiq-AAD",
];

pub fn init_tracing() {
    let silent = std::env::var("RECTIQ_SILENT")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let level = if silent { Level::WARN } else { Level::INFO };
    let _ = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .compact()
        .try_init();
}

pub fn install_panic_hook() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| {
                info.payload()
                    .downcast_ref::<String>()
                    .map(std::string::String::as_str)
            })
            .unwrap_or("panic");
        let mut red = msg.replace("Bearer ", "Bearer [REDACTED]");
        red = red.replace("X-Admin-Key:", "X-Admin-Key: [REDACTED]");
        eprintln!("rectiq-cli: panic (redacted): {red}");
        (prev)(info);
    }));
}

/// Replace sensitive header values (best-effort) before logging.
#[must_use]
pub fn redact_headers(mut h: reqwest::header::HeaderMap) -> reqwest::header::HeaderMap {
    for k in REDACT_KEYS {
        if h.contains_key(*k) {
            let val = if *k == "Authorization" {
                "Bearer [REDACTED]"
            } else {
                "[REDACTED]"
            };
            h.insert(*k, reqwest::header::HeaderValue::from_static(val));
        }
    }
    h
}

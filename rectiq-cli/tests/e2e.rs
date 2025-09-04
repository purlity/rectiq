// rectiq-cli/tests/e2e.rs
use rectiq_cli::run;
use rectiq_test_support::mock_divine_key::{spawn, divine_key_base as base_url};
use std::fs;

fn e2e_enabled() -> bool {
    std::env::var_os("RECTIQ_E2E").is_some()
}

struct EnvGuard {
    key: String,
    old: Option<String>,
}

impl EnvGuard {
    fn set(key: &str, val: &str) -> Self {
        let old = std::env::var(key).ok();
        unsafe { std::env::set_var(key, val) };
        Self {
            key: key.to_owned(),
            old,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(ref v) = self.old {
            unsafe { std::env::set_var(&self.key, v) };
        } else {
            unsafe { std::env::remove_var(&self.key) };
        }
    }
}

#[test]
fn test_end_to_end_fixing() {
    if !e2e_enabled() {
        eprintln!("skipping CLI e2e (set RECTIQ_E2E=1 to run)");
        return;
    }

    let input =
        fs::read_to_string("tests/fixtures/single_case.json").expect("Failed to read input JSON");

    let api_key = "free-tier";
    let user_id = api_key;

    let _api_key_guard = EnvGuard::set("RECTIQ_API_KEY", api_key);

    // Start mock divine-key service for hermetic tests
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (addr, _jh) = rt.block_on(async { spawn() });
    let dk_url = base_url(&addr);

    // Point CLI to test/local services
    let _env_guard = EnvGuard::set("RECTIQ_ENV", "dev");
    let _dk_url_guard = EnvGuard::set("DIVINE_KEY_URL", &dk_url);
    let _dk_base_guard = EnvGuard::set("RECTIQ_DIVINE_KEY_BASE", &addr);

    let result = run(&input, user_id).expect("Fixer CLI run failed");

    let expected = fs::read_to_string("tests/fixtures/expected_output.json")
        .expect("Failed to read expected fixed JSON");

    assert_eq!(result.trim(), expected.trim());
}

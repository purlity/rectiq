// rectiq-cli/tests/e2e_smoke.rs
// End-to-end smoke tests for Rectiq CLI library.
use rectiq_cli::{run, scan};
use rectiq_test_support::mock_divine_key::{spawn, divine_key_base as base_url};
use rectiq_types::{Kind, SketchNode};

fn e2e_enabled() -> bool {
    std::env::var_os("RECTIQ_E2E").is_some()
}

#[test]
fn scan_detects_basic_errors() {
    // Each tuple is (description, json_source, expected_kind)
    let cases = vec![
        ("trailing comma in array", "[1, 2, ]", Kind::TrailingComma),
        (
            "missing comma between items",
            "[1 2]",
            Kind::MissingCommaBetweenItem,
        ),
    ];

    for (desc, src, expected) in cases {
        let sketches: Vec<SketchNode<'static>> = scan(src);
        let has_kind = sketches.iter().any(|sk| *sk.kind() == expected);
        assert!(
            has_kind,
            "Case '{desc}' did not detect expected kind {expected:?}.\nGot: {sketches:?}"
        );
    }
}

#[test]
fn run_pipeline_returns_output() {
    if !e2e_enabled() {
        eprintln!("skipping CLI e2e (set RECTIQ_E2E=1 to run)");
        return;
    }
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (addr, _jh) = rt.block_on(async { spawn() });
    let dk_url = base_url(&addr);
    unsafe {
        std::env::set_var("DIVINE_KEY_URL", &dk_url);
        std::env::set_var("RECTIQ_DIVINE_KEY_BASE", &addr);
        std::env::set_var("RECTIQ_ENV", "dev");
        std::env::set_var("RECTIQ_API_KEY", "free-tier");
    }

    let input = r"[ 1, 2, ]";
    // Note: replace "dummy-user" with a real or test-safe user_id if your API enforces it.
    let result = run(input, "dummy-user");
    assert!(result.is_ok(), "Full run() pipeline failed: {result:?}");
}

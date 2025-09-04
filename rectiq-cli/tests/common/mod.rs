// Shared test helpers for rectiq-cli tests.

// This module provides common test utilities and macros for use in all test modules.
use rectiq_cli::scan;
use rectiq_types::{Kind, SketchNode, SketchPayload};
use std::fs;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

/// Return (start,end) spans for all matches of the given kind in the given **source string**.
pub fn spans_for_source(kind: Kind, source: &str) -> Vec<(usize, usize)> {
    let sketches: Vec<SketchNode<'static>> = scan(source);
    let mut out: Vec<(usize, usize)> = Vec::new();
    for sk in sketches {
        if *sk.kind() == kind
            && let SketchPayload::Spans(spans) = sk.payload
        {
            for s in spans {
                out.push((s.span.start, s.span.end));
            }
        }
    }
    out
}

/// Read a fixture under `tests/fixtures/` and return spans for the given kind.
pub fn spans_for_fixture(kind: Kind, rel_path: &str) -> Vec<(usize, usize)> {
    let full = format!("tests/fixtures/{rel_path}");
    let input = fs::read_to_string(&full).expect("Failed to read fixture");
    spans_for_source(kind, &input)
}

#[allow(dead_code)]
/// Assert helper for **source string** inputs.
pub fn assert_spans_eq(kind: Kind, source: &str, expected: &[(usize, usize)]) {
    let mut got = spans_for_source(kind, source);
    let mut exp = expected.to_vec();
    got.sort_unstable();
    exp.sort_unstable();
    assert_eq!(
        got, exp,
        "kind={kind:?} spans mismatch.\n  got: {got:?}\n  exp: {exp:?}"
    );
}

#[allow(dead_code)]
/// Assert helper for **fixture file** inputs.
pub fn assert_spans_eq_fixture(kind: Kind, rel_path: &str, expected: &[(usize, usize)]) {
    let mut got = spans_for_fixture(kind, rel_path);
    let mut exp = expected.to_vec();
    got.sort_unstable();
    exp.sort_unstable();
    assert_eq!(
        got, exp,
        "(fixture {rel_path}) kind={kind:?} spans mismatch.\n  got: {got:?}\n  exp: {exp:?}"
    );
}

/// Macro to assert spans against a **source string**.
#[macro_export]
macro_rules! expect_spans {
    ($kind:expr, $src:expr, [$([$s:expr, $e:expr]),* $(,)?]) => {{
        let expected: Vec<(usize, usize)> = vec![$(($s, $e)),*];
        $crate::common::assert_spans_eq($kind, $src, &expected);
    }};
}

/// Macro to assert spans against a **fixture path under tests/fixtures/**.
#[macro_export]
macro_rules! expect_spans_fixture {
    // Accept tuple pairs: [(s,e), (s,e)]
    ($kind:expr, $rel:expr, [$(($s:expr, $e:expr)),* $(,)?]) => {{
        let expected: Vec<(usize, usize)> = vec![$(($s, $e)),*];
        $crate::common::assert_spans_eq_fixture($kind, $rel, &expected);
    }};
    // Accept bracket pairs: [[s,e], [s,e]]
    ($kind:expr, $rel:expr, [$([$s:expr, $e:expr]),* $(,)?]) => {{
        let expected: Vec<(usize, usize)> = vec![$(($s, $e)),*];
        $crate::common::assert_spans_eq_fixture($kind, $rel, &expected);
    }};
}

#[macro_export]
macro_rules! expect_nonempty_fixture {
    ($kind:expr, $rel:expr) => {{
        let spans = $crate::common::spans_for_fixture($kind, $rel);
        assert!(
            !spans.is_empty(),
            "(fixture {}) kind={:?}: expected at least one span, found none",
            $rel,
            $kind
        );
    }};
}

#[allow(dead_code)]
/// Prepare test headers with a bearer token for test HTTP requests.
pub fn prepare_headers_with_token_basic(raw_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let v = format!("Bearer {raw_token}");
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&v).expect("valid auth header"),
    );
    headers
}

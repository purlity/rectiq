// tests/test_improper_nesting.rs

mod common;
use rectiq_types::Kind;

#[test]
fn improper_nesting_positive() {
    expect_nonempty_fixture!(
        Kind::ImproperNesting,
        "sketches/improper_nesting/improper_nesting_positive.json"
    );
}

#[test]
fn improper_nesting_clean() {
    let spans = common::spans_for_fixture(
        Kind::ImproperNesting,
        "sketches/improper_nesting/improper_nesting_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn improper_nesting_positive_min() {
    expect_nonempty_fixture!(
        Kind::ImproperNesting,
        "sketches/improper_nesting/improper_nesting_positive_min.json"
    );
}

#[test]
fn improper_nesting_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::ImproperNesting,
            "sketches/improper_nesting/improper_nesting_clean_min.json"
        )
        .is_empty()
    );
}

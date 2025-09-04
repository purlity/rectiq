// tests/test_leading_comma.rs

mod common;
use rectiq_types::Kind;

#[test]
fn leading_comma_positive() {
    expect_nonempty_fixture!(
        Kind::LeadingComma,
        "sketches/leading_comma/leading_comma_positive.json"
    );
}

#[test]
fn leading_comma_clean() {
    let spans = common::spans_for_fixture(
        Kind::LeadingComma,
        "sketches/leading_comma/leading_comma_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn leading_comma_positive_min() {
    expect_nonempty_fixture!(
        Kind::LeadingComma,
        "sketches/leading_comma/leading_comma_positive_min.json"
    );
}

#[test]
fn leading_comma_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::LeadingComma,
            "sketches/leading_comma/leading_comma_clean_min.json"
        )
        .is_empty()
    );
}

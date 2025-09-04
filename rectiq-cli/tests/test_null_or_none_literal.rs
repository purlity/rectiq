// tests/test_null_or_none_literal.rs

mod common;
use rectiq_types::Kind;

#[test]
fn null_or_none_literal_positive() {
    expect_nonempty_fixture!(
        Kind::NullOrNoneLiteral,
        "sketches/null_or_none_literal/null_or_none_literal_positive.json"
    );
}

#[test]
fn null_or_none_literal_clean() {
    let spans = common::spans_for_fixture(
        Kind::NullOrNoneLiteral,
        "sketches/null_or_none_literal/null_or_none_literal_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn null_or_none_literal_positive_min() {
    expect_nonempty_fixture!(
        Kind::NullOrNoneLiteral,
        "sketches/null_or_none_literal/null_or_none_literal_positive_min.json"
    );
}

#[test]
fn null_or_none_literal_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::NullOrNoneLiteral,
            "sketches/null_or_none_literal/null_or_none_literal_clean_min.json"
        )
        .is_empty()
    );
}

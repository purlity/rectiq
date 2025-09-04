// tests/test_invalid_number_format.rs

mod common;
use rectiq_types::Kind;

#[test]
fn invalid_number_format_positive() {
    expect_nonempty_fixture!(
        Kind::InvalidNumberFormat,
        "sketches/invalid_number_format/invalid_number_format_positive.json"
    );
}

#[test]
fn invalid_number_format_clean() {
    let spans = common::spans_for_fixture(
        Kind::InvalidNumberFormat,
        "sketches/invalid_number_format/invalid_number_format_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn invalid_number_format_positive_min() {
    expect_nonempty_fixture!(
        Kind::InvalidNumberFormat,
        "sketches/invalid_number_format/invalid_number_format_positive_min.json"
    );
}

#[test]
fn invalid_number_format_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::InvalidNumberFormat,
            "sketches/invalid_number_format/invalid_number_format_clean_min.json"
        )
        .is_empty()
    );
}

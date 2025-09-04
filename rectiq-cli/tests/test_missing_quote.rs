// tests/test_missing_quote.rs

mod common;
use rectiq_types::Kind;

#[test]
fn missing_quote_positive() {
    expect_nonempty_fixture!(
        Kind::MissingQuote,
        "sketches/missing_quote/missing_quote_positive.json"
    );
}

#[test]
fn missing_quote_clean() {
    let spans = common::spans_for_fixture(
        Kind::MissingQuote,
        "sketches/missing_quote/missing_quote_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn missing_quote_positive_min() {
    expect_nonempty_fixture!(
        Kind::MissingQuote,
        "sketches/missing_quote/missing_quote_positive_min.json"
    );
}

#[test]
fn missing_quote_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::MissingQuote,
            "sketches/missing_quote/missing_quote_clean_min.json"
        )
        .is_empty()
    );
}

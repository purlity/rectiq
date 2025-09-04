// tests/test_unescaped_quote.rs

mod common;
use rectiq_types::Kind;

#[test]
fn unescaped_quote_positive() {
    expect_nonempty_fixture!(
        Kind::UnescapedQuote,
        "sketches/unescaped_quote/unescaped_quote_positive.json"
    );
}

#[test]
fn unescaped_quote_clean() {
    let spans = common::spans_for_fixture(
        Kind::UnescapedQuote,
        "sketches/unescaped_quote/unescaped_quote_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn unescaped_quote_positive_min() {
    expect_nonempty_fixture!(
        Kind::UnescapedQuote,
        "sketches/unescaped_quote/unescaped_quote_positive_min.json"
    );
}

#[test]
fn unescaped_quote_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::UnescapedQuote,
            "sketches/unescaped_quote/unescaped_quote_clean_min.json"
        )
        .is_empty()
    );
}

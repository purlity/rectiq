// tests/test_excess_whitespace_or_newline.rs

mod common;
use rectiq_types::Kind;

#[test]
fn excess_whitespace_or_newline_positive() {
    expect_nonempty_fixture!(
        Kind::ExcessWhitespaceOrNewline,
        "sketches/excess_whitespace_or_newline/excess_whitespace_or_newline_positive.json"
    );
}

#[test]
fn excess_whitespace_or_newline_clean() {
    let spans = common::spans_for_fixture(
        Kind::ExcessWhitespaceOrNewline,
        "sketches/excess_whitespace_or_newline/excess_whitespace_or_newline_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn excess_whitespace_or_newline_positive_min() {
    expect_nonempty_fixture!(
        Kind::ExcessWhitespaceOrNewline,
        "sketches/excess_whitespace_or_newline/excess_whitespace_or_newline_positive_min.json"
    );
}

#[test]
fn excess_whitespace_or_newline_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::ExcessWhitespaceOrNewline,
            "sketches/excess_whitespace_or_newline/excess_whitespace_or_newline_clean_min.json"
        )
        .is_empty()
    );
}

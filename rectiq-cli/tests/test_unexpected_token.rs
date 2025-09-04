// tests/test_unexpected_token.rs

mod common;
use rectiq_types::Kind;

#[test]
fn unexpected_token_positive() {
    expect_nonempty_fixture!(
        Kind::UnexpectedToken,
        "sketches/unexpected_token/unexpected_token_positive.json"
    );
}

#[test]
fn unexpected_token_clean() {
    let spans = common::spans_for_fixture(
        Kind::UnexpectedToken,
        "sketches/unexpected_token/unexpected_token_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn unexpected_token_ignored_in_comment() {
    assert!(
        common::spans_for_fixture(
            Kind::UnexpectedToken,
            "sketches/unexpected_token/unexpected_token_comment.json"
        )
        .is_empty()
    );
}

#[test]
fn unexpected_token_ignored_in_key() {
    assert!(
        common::spans_for_fixture(
            Kind::UnexpectedToken,
            "sketches/unexpected_token/unexpected_token_key.json"
        )
        .is_empty()
    );
}

#[test]
fn unexpected_token_positive_min() {
    expect_nonempty_fixture!(
        Kind::UnexpectedToken,
        "sketches/unexpected_token/unexpected_token_positive_min.json"
    );
}

#[test]
fn unexpected_token_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::UnexpectedToken,
            "sketches/unexpected_token/unexpected_token_clean_min.json"
        )
        .is_empty()
    );
}

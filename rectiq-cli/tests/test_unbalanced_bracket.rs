// tests/test_unbalanced_bracket.rs

mod common;
use rectiq_types::Kind;

#[test]
fn unbalanced_bracket_positive() {
    expect_nonempty_fixture!(
        Kind::UnbalancedBracket,
        "sketches/unbalanced_bracket/unbalanced_bracket_positive.json"
    );
}

#[test]
fn unbalanced_bracket_clean() {
    let spans = common::spans_for_fixture(
        Kind::UnbalancedBracket,
        "sketches/unbalanced_bracket/unbalanced_bracket_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn unbalanced_bracket_positive_min() {
    expect_nonempty_fixture!(
        Kind::UnbalancedBracket,
        "sketches/unbalanced_bracket/unbalanced_bracket_positive_min.json"
    );
}

#[test]
fn unbalanced_bracket_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::UnbalancedBracket,
            "sketches/unbalanced_bracket/unbalanced_bracket_clean_min.json"
        )
        .is_empty()
    );
}

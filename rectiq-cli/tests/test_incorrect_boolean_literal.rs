// tests/test_incorrect_boolean_literal.rs

mod common;
use rectiq_types::Kind;

#[test]
fn incorrect_boolean_literal_positive() {
    expect_nonempty_fixture!(
        Kind::IncorrectBooleanLiteral,
        "sketches/incorrect_boolean_literal/incorrect_boolean_literal_positive.json"
    );
}

#[test]
fn incorrect_boolean_literal_clean() {
    let spans = common::spans_for_fixture(
        Kind::IncorrectBooleanLiteral,
        "sketches/incorrect_boolean_literal/incorrect_boolean_literal_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn incorrect_boolean_literal_positive_min() {
    expect_nonempty_fixture!(
        Kind::IncorrectBooleanLiteral,
        "sketches/incorrect_boolean_literal/incorrect_boolean_literal_positive_min.json"
    );
}

#[test]
fn incorrect_boolean_literal_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::IncorrectBooleanLiteral,
            "sketches/incorrect_boolean_literal/incorrect_boolean_literal_clean_min.json"
        )
        .is_empty()
    );
}

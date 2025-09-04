// tests/test_invalid_escape_sequence.rs

mod common;
use rectiq_types::Kind;

#[test]
fn invalid_escape_sequence_positive() {
    expect_nonempty_fixture!(
        Kind::InvalidEscapeSequence,
        "sketches/invalid_escape_sequence/invalid_escape_sequence_positive.json"
    );
}

#[test]
fn invalid_escape_sequence_clean() {
    let spans = common::spans_for_fixture(
        Kind::InvalidEscapeSequence,
        "sketches/invalid_escape_sequence/invalid_escape_sequence_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn invalid_escape_sequence_positive_min() {
    expect_nonempty_fixture!(
        Kind::InvalidEscapeSequence,
        "sketches/invalid_escape_sequence/invalid_escape_sequence_positive_min.json"
    );
}

#[test]
fn invalid_escape_sequence_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::InvalidEscapeSequence,
            "sketches/invalid_escape_sequence/invalid_escape_sequence_clean_min.json"
        )
        .is_empty()
    );
}

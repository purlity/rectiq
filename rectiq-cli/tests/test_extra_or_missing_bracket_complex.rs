// tests/test_extra_or_missing_bracket_complex.rs

mod common;
use rectiq_types::Kind;

#[test]
fn extra_or_missing_bracket_complex_positive() {
    expect_nonempty_fixture!(
        Kind::ExtraOrMissingBracketComplex,
        "sketches/extra_or_missing_bracket_complex/extra_or_missing_bracket_complex_positive.json"
    );
}

#[test]
fn extra_or_missing_bracket_complex_clean() {
    let spans = common::spans_for_fixture(
        Kind::ExtraOrMissingBracketComplex,
        "sketches/extra_or_missing_bracket_complex/extra_or_missing_bracket_complex_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn extra_or_missing_bracket_complex_positive_min() {
    expect_nonempty_fixture!(
        Kind::ExtraOrMissingBracketComplex,
        "sketches/extra_or_missing_bracket_complex/extra_or_missing_bracket_complex_positive_min.json"
    );
}

#[test]
fn extra_or_missing_bracket_complex_clean_min() {
    assert!(common::spans_for_fixture(
        Kind::ExtraOrMissingBracketComplex,
        "sketches/extra_or_missing_bracket_complex/extra_or_missing_bracket_complex_clean_min.json"
    )
    .is_empty());
}

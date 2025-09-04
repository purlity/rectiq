// tests/test_invalid_character.rs

mod common;
use rectiq_types::Kind;

#[test]
fn invalid_character_positive() {
    expect_nonempty_fixture!(
        Kind::InvalidCharacter,
        "sketches/invalid_character/invalid_character_positive.json"
    );
}

#[test]
fn invalid_character_clean() {
    let spans = common::spans_for_fixture(
        Kind::InvalidCharacter,
        "sketches/invalid_character/invalid_character_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn invalid_character_positive_min() {
    expect_nonempty_fixture!(
        Kind::InvalidCharacter,
        "sketches/invalid_character/invalid_character_positive_min.json"
    );
}

#[test]
fn invalid_character_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::InvalidCharacter,
            "sketches/invalid_character/invalid_character_clean_min.json"
        )
        .is_empty()
    );
}

// tests/test_duplicate_key.rs

mod common;
use rectiq_types::Kind;

#[test]
fn duplicate_key_positive() {
    expect_nonempty_fixture!(
        Kind::DuplicateKey,
        "sketches/duplicate_key/duplicate_key_positive.json"
    );
}

#[test]
fn duplicate_key_clean() {
    let spans = common::spans_for_fixture(
        Kind::DuplicateKey,
        "sketches/duplicate_key/duplicate_key_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn duplicate_key_positive_min() {
    expect_nonempty_fixture!(
        Kind::DuplicateKey,
        "sketches/duplicate_key/duplicate_key_positive_min.json"
    );
}

#[test]
fn duplicate_key_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::DuplicateKey,
            "sketches/duplicate_key/duplicate_key_clean_min.json"
        )
        .is_empty()
    );
}

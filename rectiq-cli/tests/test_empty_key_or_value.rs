// tests/test_empty_key_or_value.rs

mod common;
use rectiq_types::Kind;

#[test]
fn empty_key_or_value_positive() {
    expect_nonempty_fixture!(
        Kind::EmptyKeyOrValue,
        "sketches/empty_key_or_value/empty_key_or_value_positive.json"
    );
}

#[test]
fn empty_key_or_value_clean() {
    let spans = common::spans_for_fixture(
        Kind::EmptyKeyOrValue,
        "sketches/empty_key_or_value/empty_key_or_value_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn empty_key_or_value_positive_min() {
    expect_nonempty_fixture!(
        Kind::EmptyKeyOrValue,
        "sketches/empty_key_or_value/empty_key_or_value_positive_min.json"
    );
}

#[test]
fn empty_key_or_value_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::EmptyKeyOrValue,
            "sketches/empty_key_or_value/empty_key_or_value_clean_min.json"
        )
        .is_empty()
    );
}

// tests/test_mixed_type_in_array.rs

mod common;
use rectiq_types::Kind;

#[test]
fn mixed_type_in_array_positive() {
    expect_nonempty_fixture!(
        Kind::MixedTypeInArray,
        "sketches/mixed_type_in_array/mixed_type_in_array_positive.json"
    );
}

#[test]
fn mixed_type_in_array_clean() {
    let spans = common::spans_for_fixture(
        Kind::MixedTypeInArray,
        "sketches/mixed_type_in_array/mixed_type_in_array_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn mixed_type_in_array_positive_min() {
    expect_nonempty_fixture!(
        Kind::MixedTypeInArray,
        "sketches/mixed_type_in_array/mixed_type_in_array_positive_min.json"
    );
}

#[test]
fn mixed_type_in_array_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::MixedTypeInArray,
            "sketches/mixed_type_in_array/mixed_type_in_array_clean_min.json"
        )
        .is_empty()
    );
}

// tests/test_overly_large_number.rs

mod common;
use rectiq_types::Kind;

#[test]
fn overly_large_number_positive() {
    expect_nonempty_fixture!(
        Kind::OverlyLargeNumber,
        "sketches/overly_large_number/overly_large_number_positive.json"
    );
}

#[test]
fn overly_large_number_clean() {
    let spans = common::spans_for_fixture(
        Kind::OverlyLargeNumber,
        "sketches/overly_large_number/overly_large_number_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn overly_large_number_positive_min() {
    expect_nonempty_fixture!(
        Kind::OverlyLargeNumber,
        "sketches/overly_large_number/overly_large_number_positive_min.json"
    );
}

#[test]
fn overly_large_number_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::OverlyLargeNumber,
            "sketches/overly_large_number/overly_large_number_clean_min.json"
        )
        .is_empty()
    );
}

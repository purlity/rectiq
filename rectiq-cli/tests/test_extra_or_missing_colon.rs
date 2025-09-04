// tests/test_extra_or_missing_colon.rs

mod common;
use rectiq_types::Kind;

#[test]
fn extra_or_missing_colon_positive() {
    expect_nonempty_fixture!(
        Kind::ExtraOrMissingColon,
        "sketches/extra_or_missing_colon/extra_or_missing_colon_positive.json"
    );
}

#[test]
fn extra_or_missing_colon_clean() {
    let spans = common::spans_for_fixture(
        Kind::ExtraOrMissingColon,
        "sketches/extra_or_missing_colon/extra_or_missing_colon_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn extra_or_missing_colon_positive_min() {
    expect_nonempty_fixture!(
        Kind::ExtraOrMissingColon,
        "sketches/extra_or_missing_colon/extra_or_missing_colon_positive_min.json"
    );
}

#[test]
fn extra_or_missing_colon_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::ExtraOrMissingColon,
            "sketches/extra_or_missing_colon/extra_or_missing_colon_clean_min.json"
        )
        .is_empty()
    );
}

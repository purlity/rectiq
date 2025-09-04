// tests/test_missing_comma_between_item.rs

mod common;
use rectiq_types::Kind;

#[test]
fn missing_comma_between_item_positive() {
    expect_nonempty_fixture!(
        Kind::MissingCommaBetweenItem,
        "sketches/missing_comma_between_item/missing_comma_between_item_positive.json"
    );
}

#[test]
fn missing_comma_between_item_clean() {
    let spans = common::spans_for_fixture(
        Kind::MissingCommaBetweenItem,
        "sketches/missing_comma_between_item/missing_comma_between_item_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn missing_comma_between_item_positive_min() {
    expect_nonempty_fixture!(
        Kind::MissingCommaBetweenItem,
        "sketches/missing_comma_between_item/missing_comma_between_item_positive_min.json"
    );
}

#[test]
fn missing_comma_between_item_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::MissingCommaBetweenItem,
            "sketches/missing_comma_between_item/missing_comma_between_item_clean_min.json"
        )
        .is_empty()
    );
}

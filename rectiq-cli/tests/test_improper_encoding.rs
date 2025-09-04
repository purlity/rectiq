// tests/test_improper_encoding.rs

mod common;
use rectiq_types::Kind;

#[test]
fn improper_encoding_positive() {
    expect_nonempty_fixture!(
        Kind::ImproperEncoding,
        "sketches/improper_encoding/improper_encoding_positive.json"
    );
}

#[test]
fn improper_encoding_clean() {
    let spans = common::spans_for_fixture(
        Kind::ImproperEncoding,
        "sketches/improper_encoding/improper_encoding_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn improper_encoding_positive_min() {
    expect_nonempty_fixture!(
        Kind::ImproperEncoding,
        "sketches/improper_encoding/improper_encoding_positive_min.json"
    );
}

#[test]
fn improper_encoding_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::ImproperEncoding,
            "sketches/improper_encoding/improper_encoding_clean_min.json"
        )
        .is_empty()
    );
}

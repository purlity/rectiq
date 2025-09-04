// tests/test_comment_in_json.rs

mod common;
use rectiq_types::Kind;

#[test]
fn comment_in_json_positive() {
    expect_nonempty_fixture!(
        Kind::CommentInJSON,
        "sketches/comment_in_json/comment_in_json_positive.json"
    );
}

#[test]
fn comment_in_json_clean() {
    let spans = common::spans_for_fixture(
        Kind::CommentInJSON,
        "sketches/comment_in_json/comment_in_json_clean.json",
    );
    assert!(spans.is_empty(), "Expected no spans, got {spans:?}");
}

#[test]
fn comment_in_json_positive_min() {
    expect_nonempty_fixture!(
        Kind::CommentInJSON,
        "sketches/comment_in_json/comment_in_json_positive_min.json"
    );
}

#[test]
fn comment_in_json_clean_min() {
    assert!(
        common::spans_for_fixture(
            Kind::CommentInJSON,
            "sketches/comment_in_json/comment_in_json_clean_min.json"
        )
        .is_empty()
    );
}

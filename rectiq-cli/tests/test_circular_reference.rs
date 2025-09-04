// rectiq-cli/tests/test_circular_reference.rs
// Scan-only tests for CircularReference using Kind enum and shared util helpers.

use rectiq_types::Kind;
mod common;

#[test]
fn scan_only_circular_reference_simple() {
    // Only require that some span is found.
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_reference_simple.json",
        []
    );
}

#[test]
fn scan_only_circular_self_reference() {
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_self_reference.json",
        []
    );
}

#[test]
fn scan_only_circular_indirect_reference() {
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_indirect_reference.json",
        []
    );
}

#[test]
fn scan_only_circular_array_reference() {
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_array_reference.json",
        []
    );
}

#[test]
fn scan_only_circular_nested_objects() {
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_nested_objects.json",
        []
    );
}

#[test]
fn scan_only_circular_with_mixed_types() {
    expect_spans_fixture!(
        Kind::CircularReference,
        "sketches/circular_reference/circular_with_mixed_types.json",
        []
    );
}

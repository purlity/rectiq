use rectiq_cli::pipeline::{build_skeleton, lex};

#[test]
fn skeleton_obj_pairs_and_arrays() {
    let src = "{\"a\":1,\"b\":[10,20]}";
    let tokens = lex(src);
    let skel = build_skeleton(src, &tokens);
    assert_eq!(skel.obj_pairs.len(), 2);
    assert_eq!(skel.arr_elems.len(), 2);
}

#[test]
fn skeleton_bracket_mismatch() {
    let src = "{]";
    let tokens = lex(src);
    let skel = build_skeleton(src, &tokens);
    assert_eq!(skel.bracket_mismatches, vec![1]);
}

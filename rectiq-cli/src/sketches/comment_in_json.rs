// rectiq-cli/src/sketches/comment_in_json.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect any comment regions within the input. JSON does not allow comments,
///
/// so every `TokKind::Comment` token becomes a span unless it resides within a
/// string (which the lattice prevents).
pub struct CommentInJsonSketcher;

impl Default for CommentInJsonSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CommentInJsonSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for CommentInJsonSketcher {
    fn name(&self) -> &'static str {
        "CommentInJSON"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, _c: char, _offset: usize) {}

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;
        let mut spans = Vec::new();

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.kind == TokKind::Comment && lattice.class_for(tok.start) == RegionClass::Comment
            {
                let (parent_keys, depth) = skel.path_at(&tokens, input, idx);
                spans.push(SpanContext::new(
                    input,
                    tok.start,
                    tok.end,
                    depth,
                    parent_keys,
                ));
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::CommentInJSON,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

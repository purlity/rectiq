// rectiq-cli/src/sketches/invalid_character.rs
use crate::{TokenSketcher, pipeline::TokKind, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Emit spans for any unknown characters outside of strings and comments.
///
/// These characters do not correspond to any valid JSON token and are not
/// part of another specialized sketcher.
pub struct InvalidCharacterSketcher {
    maybe_has_control: bool,
}

impl Default for InvalidCharacterSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl InvalidCharacterSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_control: false,
        }
    }
}

impl TokenSketcher for InvalidCharacterSketcher {
    fn name(&self) -> &'static str {
        "InvalidCharacter"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if !c.is_ascii() || (c.is_control() && !c.is_whitespace()) {
            self.maybe_has_control = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_control {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let input = &pool.input;
        let mut spans = Vec::new();

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.kind != TokKind::Unknown {
                continue;
            }
            // Unknown tokens are single-byte per lexer; avoid UTF-8 slicing here.
            let b = input.as_bytes()[tok.start];
            if b == b'"' || b.is_ascii_alphabetic() {
                continue; // handled by other sketchers
            }
            let (parent_keys, depth) = skel.path_at(&tokens, input, idx);
            spans.push(SpanContext::new(
                input,
                tok.start,
                tok.end,
                depth,
                parent_keys,
            ));
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::InvalidCharacter,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

// rectiq-cli/src/sketches/unescaped_quote.rs
use crate::{TokenSketcher, pipeline::TokKind, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect stray quote characters that are not part of a valid string literal.
pub struct UnescapedQuoteSketcher {
    maybe_has_quote: bool,
}

impl Default for UnescapedQuoteSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UnescapedQuoteSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_quote: false,
        }
    }
}

impl TokenSketcher for UnescapedQuoteSketcher {
    fn name(&self) -> &'static str {
        "UnescapedQuote"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == '"' {
            self.maybe_has_quote = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_quote {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let input = &pool.input;
        let mut spans = Vec::new();

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.kind == TokKind::Unknown && input.as_bytes()[tok.start] == b'"' {
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
                kind: Kind::UnescapedQuote,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

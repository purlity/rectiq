// rectiq-cli/src/sketches/unexpected_token.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, Skeleton, TokKind, Token},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, Span, SpanContext,
    span_utils::merge_adjacent_single_char_spans,
};

pub struct UnexpectedTokenSketcher {}

impl Default for UnexpectedTokenSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UnexpectedTokenSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    fn make_span<'a>(
        skel: &Skeleton<'static>,
        tokens: &[Token],
        idx: usize,
        tok: &Token,
        input: &'a str,
    ) -> SpanContext<'a> {
        let (parent_keys, context_depth) = skel.path_at(tokens, input, idx);
        let span = Span::new(input, tok.start, tok.end);
        SpanContext {
            span,
            context_depth,
            parent_keys,
        }
    }
}

impl TokenSketcher for UnexpectedTokenSketcher {
    fn name(&self) -> &'static str {
        "UnexpectedToken"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, _c: char, _offset: usize) {
        // Token-based pipeline: decisions are made in `finalize` using tokens + lattice.
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans: Vec<SpanContext> = Vec::new();
        // Skip the final EOF token if present by taking len()-1 safely
        for (idx, tok) in tokens
            .iter()
            .take(tokens.len().saturating_sub(1))
            .enumerate()
        {
            if tok.kind == TokKind::Unknown {
                let class = lattice.class_for(tok.start);
                if !matches!(
                    class,
                    RegionClass::Comment | RegionClass::String | RegionClass::Key
                ) {
                    spans.push(Self::make_span(&skel, &tokens, idx, tok, input));
                }
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::UnexpectedToken,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

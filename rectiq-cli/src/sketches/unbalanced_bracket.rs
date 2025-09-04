// rectiq-cli/src/sketches/unbalanced_bracket.rs
use crate::{TokenSketcher, pipeline::RegionClass, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

pub struct UnbalancedBracketSketcher {
    maybe_has_brackets: bool,
}

impl Default for UnbalancedBracketSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl UnbalancedBracketSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_brackets: false,
        }
    }
}

impl TokenSketcher for UnbalancedBracketSketcher {
    fn name(&self) -> &'static str {
        "UnbalancedBracket"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if matches!(c, '{' | '}' | '[' | ']') {
            self.maybe_has_brackets = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_brackets {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans = Vec::new();
        for byte in &skel.bracket_mismatches {
            if let Some((idx, tok)) = tokens.iter().enumerate().find(|(_, t)| t.start == *byte) {
                if matches!(
                    lattice.class_for(tok.start),
                    RegionClass::Comment | RegionClass::String
                ) {
                    continue;
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
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::UnbalancedBracket,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

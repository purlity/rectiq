// rectiq-cli/src/sketches/null_or_none_literal.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect `Null`, `NULL`, or Python-style `None` literals outside of strings.
pub struct NullOrNoneLiteralSketcher {
    maybe_has_n: bool,
}

impl Default for NullOrNoneLiteralSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl NullOrNoneLiteralSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self { maybe_has_n: false }
    }
}

impl TokenSketcher for NullOrNoneLiteralSketcher {
    fn name(&self) -> &'static str {
        "NullOrNoneLiteral"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == 'n' || c == 'N' {
            self.maybe_has_n = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_n {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;
        let mut spans = Vec::new();

        let mut i = 0;
        while i < tokens.len() {
            let tok = &tokens[i];
            if tok.kind == TokKind::Unknown
                && !matches!(
                    lattice.class_for(tok.start),
                    RegionClass::Comment | RegionClass::String
                )
            {
                let ch = input[tok.start..tok.end].chars().next().unwrap();
                if ch.is_ascii_alphabetic() {
                    let start = i;
                    let mut end = i;
                    let mut text = String::new();
                    while end < tokens.len() {
                        let t = &tokens[end];
                        if t.kind == TokKind::Unknown {
                            let c = input[t.start..t.end].chars().next().unwrap();
                            if c.is_ascii_alphabetic() {
                                text.push(c);
                                end += 1;
                                continue;
                            }
                        }
                        break;
                    }
                    let lower = text.to_ascii_lowercase();
                    let is_bad_null = lower == "null" && text != "null";
                    let is_none = lower == "none";
                    if is_bad_null || is_none {
                        let (parent_keys, depth) = skel.path_at(&tokens, input, start);
                        let span_start = tokens[start].start;
                        let span_end = tokens[end - 1].end;
                        spans.push(SpanContext::new(
                            input,
                            span_start,
                            span_end,
                            depth,
                            parent_keys,
                        ));
                    }
                    i = end;
                    continue;
                }
            }
            i += 1;
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::NullOrNoneLiteral,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

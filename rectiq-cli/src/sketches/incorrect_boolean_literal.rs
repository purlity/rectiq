// rectiq-cli/src/sketches/incorrect_boolean_literal.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect boolean literals that have incorrect casing, such as `True` or
///
/// `FALSE`. The lexer emits these as runs of `Unknown` tokens; we merge
/// contiguous alphabetic runs and compare against the valid `true`/`false`
/// literals.
pub struct IncorrectBooleanLiteralSketcher {
    maybe_has_cap_t_or_f: bool,
}

impl Default for IncorrectBooleanLiteralSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl IncorrectBooleanLiteralSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_cap_t_or_f: false,
        }
    }
}

impl TokenSketcher for IncorrectBooleanLiteralSketcher {
    fn name(&self) -> &'static str {
        "IncorrectBooleanLiteral"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if matches!(c, 'T' | 'F') {
            self.maybe_has_cap_t_or_f = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_cap_t_or_f {
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
                    if (lower == "true" || lower == "false") && text != lower {
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
                kind: Kind::IncorrectBooleanLiteral,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

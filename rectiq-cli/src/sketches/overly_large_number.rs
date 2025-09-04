// rectiq-cli/src/sketches/overly_large_number.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

pub struct OverlyLargeNumberSketcher;

impl Default for OverlyLargeNumberSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlyLargeNumberSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for OverlyLargeNumberSketcher {
    fn name(&self) -> &'static str {
        "OverlyLargeNumber"
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
            if tok.kind != TokKind::NumberLit {
                continue;
            }
            if matches!(
                lattice.class_for(tok.start),
                RegionClass::Comment | RegionClass::String
            ) {
                continue;
            }
            let slice = &input[tok.start..tok.end];
            if is_overly_large_number(slice) {
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
                kind: Kind::OverlyLargeNumber,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

fn is_overly_large_number(num_str: &str) -> bool {
    let num = num_str
        .strip_prefix('-')
        .map_or(num_str, |stripped| stripped);
    let parts: Vec<&str> = num.split(['e', 'E']).collect();
    let base = parts[0];
    let exponent = if parts.len() > 1 {
        parts[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    let base_parts: Vec<&str> = base.split('.').collect();
    let integer_part = base_parts[0];
    let int_trimmed = integer_part.trim_start_matches('0');
    let int_len = if int_trimmed.is_empty() {
        1
    } else {
        int_trimmed.len()
    };
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let magnitude = int_len as i32 + exponent;
    magnitude > 15
}

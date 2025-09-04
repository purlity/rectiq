// rectiq-cli/src/sketches/extra_or_missing_colon.rs

use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{Kind, SketchNode, SketchPayload, SpanContext};

/// Detects object entries where the colon between key and value is missing or
///
/// repeated. Analysis is based on the SUPRA pipeline skeleton to reason about
/// key/value spans and the lattice to avoid reporting inside comments or
/// strings.
pub struct ExtraOrMissingColonSketcher;

impl Default for ExtraOrMissingColonSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtraOrMissingColonSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for ExtraOrMissingColonSketcher {
    fn name(&self) -> &'static str {
        "ExtraOrMissingColon"
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

        for pair in &skel.obj_pairs {
            let gap_range = pair.key_span.1..pair.value_span.0;
            let mut colon_indices = Vec::new();
            for idx in gap_range.clone() {
                if tokens[idx].kind == TokKind::Colon {
                    colon_indices.push(idx);
                }
            }

            if colon_indices.is_empty() {
                // Missing colon
                let mut start = tokens[pair.key_span.1 - 1].end;
                let mut end = tokens[pair.value_span.0].start;

                // trim whitespace/comments from both sides
                let mut left = pair.key_span.1;
                while left < pair.value_span.0 {
                    let t = &tokens[left];
                    if matches!(t.kind, TokKind::Whitespace | TokKind::Comment) {
                        start = t.end;
                        left += 1;
                    } else {
                        break;
                    }
                }
                let mut right = pair.value_span.0;
                while right > pair.key_span.1 {
                    let t = &tokens[right - 1];
                    if matches!(t.kind, TokKind::Whitespace | TokKind::Comment) {
                        end = t.start;
                        right -= 1;
                    } else {
                        break;
                    }
                }
                if start > end {
                    end = start;
                }
                if !matches!(
                    lattice.class_for(start),
                    RegionClass::Comment | RegionClass::String
                ) {
                    let (parent_keys, depth) = skel.path_at(&tokens, input, pair.value_span.0);
                    spans.push(SpanContext::new(input, start, end, depth, parent_keys));
                }
            } else if colon_indices.len() > 1 {
                // Extra colons: highlight all colons after the first
                let start_idx = colon_indices[1];
                let end_idx = *colon_indices.last().unwrap();
                let start = tokens[start_idx].start;
                let end = tokens[end_idx].end;
                if !matches!(
                    lattice.class_for(start),
                    RegionClass::Comment | RegionClass::String
                ) {
                    let (parent_keys, depth) = skel.path_at(&tokens, input, pair.value_span.0);
                    spans.push(SpanContext::new(input, start, end, depth, parent_keys));
                }
            }
        }

        if spans.is_empty() {
            None
        } else {
            Some(SketchNode {
                kind: Kind::ExtraOrMissingColon,
                fix_hint: None,
                payload: SketchPayload::Spans(spans),
            })
        }
    }
}

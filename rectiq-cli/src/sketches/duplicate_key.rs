// rectiq-cli/src/sketches/duplicate_key.rs
use crate::{TokenSketcher, pipeline::TokKind, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};
use std::collections::HashMap;

pub struct DuplicateKeySketcher;

impl Default for DuplicateKeySketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DuplicateKeySketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for DuplicateKeySketcher {
    fn name(&self) -> &'static str {
        "DuplicateKey"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self)
    }

    fn observe(&mut self, _c: char, _offset: usize) {}

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let _lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans = Vec::new();
        let mut by_obj: HashMap<(Vec<String>, u8), HashMap<String, usize>> = HashMap::new();

        for pair in &skel.obj_pairs {
            let key_idx = pair.key_span.0;
            let key_tok = &tokens[key_idx];
            if key_tok.kind != TokKind::StringLit {
                continue;
            }
            let (parent_keys, depth) = skel.path_at(&tokens, input, key_idx);
            let obj_id = (
                parent_keys
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                depth,
            );
            let key_start = key_tok.start + 1;
            let key_end = key_tok.end.saturating_sub(1);
            if key_end < key_start || key_end > input.len() {
                continue;
            }
            let key_text = &input[key_start..key_end];
            let entry = by_obj.entry(obj_id).or_default();
            if entry.contains_key(key_text) {
                spans.push(SpanContext::new(
                    input,
                    key_tok.start,
                    key_tok.end,
                    depth,
                    parent_keys,
                ));
            } else {
                entry.insert(key_text.to_string(), key_idx);
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::DuplicateKey,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

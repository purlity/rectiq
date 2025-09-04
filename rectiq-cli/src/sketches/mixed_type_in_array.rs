// rectiq-cli/src/sketches/mixed_type_in_array.rs
use crate::{
    pipeline::{
        TokKind,
        TokKind::{LBrace, LBracket, RBrace, RBracket},
        Token,
    },
    types::pool::LocalShapePool,
    TokenSketcher,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VKind {
    Str,
    Num,
    True,
    False,
    Null,
    Obj,
    Arr,
    Other,
}

pub struct MixedTypeInArraySketcher {
    maybe_has_array: bool,
}

impl Default for MixedTypeInArraySketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MixedTypeInArraySketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_array: false,
        }
    }
}

impl TokenSketcher for MixedTypeInArraySketcher {
    fn name(&self) -> &'static str {
        "MixedTypeInArray"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == '[' {
            self.maybe_has_array = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_array {
            return None;
        }
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let _lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut first_kind: HashMap<(usize, Vec<String>, u8), VKind> = HashMap::new();
        let mut spans = Vec::new();

        for elem in &skel.arr_elems {
            let first_idx = elem.span.0;
            let tok = &tokens[first_idx];
            let kind = match tok.kind {
                TokKind::StringLit => VKind::Str,
                TokKind::NumberLit => VKind::Num,
                TokKind::True => VKind::True,
                TokKind::False => VKind::False,
                TokKind::Null => VKind::Null,
                TokKind::LBrace => VKind::Obj,
                TokKind::LBracket => VKind::Arr,
                _ => VKind::Other,
            };
            if matches!(kind, VKind::Other) {
                continue;
            }
            // find array start byte for uniqueness
            let arr_start = find_array_start(&tokens, first_idx);
            let (parent_keys, depth) = skel.path_at(&tokens, input, first_idx);
            let id = (
                arr_start,
                parent_keys
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                depth,
            );
            match first_kind.get(&id) {
                None => {
                    first_kind.insert(id, kind);
                }
                Some(&k) if k != kind => {
                    spans.push(SpanContext::new(
                        input,
                        tok.start,
                        tok.end,
                        depth,
                        parent_keys,
                    ));
                }
                _ => {}
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::MixedTypeInArray,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

fn find_array_start(tokens: &[Token], mut idx: usize) -> usize {
    let mut depth = 0;
    while idx > 0 {
        idx -= 1;
        match tokens[idx].kind {
            RBracket | RBrace => depth += 1,
            LBracket => {
                if depth == 0 {
                    return tokens[idx].start;
                }
                depth -= 1;
            }
            LBrace => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {}
        }
    }
    0
}

// rectiq-cli/src/sketches/excess_whitespace_or_newline.rs
use crate::{TokenSketcher, pipeline::RegionClass, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};
use std::collections::HashMap;

pub struct ExcessWhitespaceOrNewlineSketcher;

impl Default for ExcessWhitespaceOrNewlineSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ExcessWhitespaceOrNewlineSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for ExcessWhitespaceOrNewlineSketcher {
    fn name(&self) -> &'static str {
        "ExcessWhitespaceOrNewline"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self)
    }

    fn observe(&mut self, _c: char, _offset: usize) {}

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut index_map = HashMap::new();
        for (i, t) in tokens.iter().enumerate() {
            index_map.insert(t.start, i);
        }

        let mut spans = Vec::new();
        for region in &lattice.regions {
            if region.class != RegionClass::Gap {
                continue;
            }
            if region.end == input.len() {
                continue;
            }
            let slice = &input[region.start..region.end];
            let Some(&tok_idx) = index_map.get(&region.start) else {
                continue;
            };
            let (parent_keys, depth) = skel.path_at(&tokens, input, tok_idx);
            let mut offset = region.start;
            let mut empty_line_streak = 0;
            for line in slice.split_inclusive('\n') {
                let trimmed = line.trim_end_matches('\n');
                let line_len = trimmed.len();
                if trimmed.ends_with(' ') || trimmed.ends_with('\t') {
                    let ws_start = line_len
                        - trimmed
                            .chars()
                            .rev()
                            .take_while(|c| *c == ' ' || *c == '\t')
                            .count();
                    spans.push(SpanContext::new(
                        input,
                        offset + ws_start,
                        offset + line_len,
                        depth,
                        parent_keys.clone(),
                    ));
                }
                let mut run_start: Option<usize> = None;
                for (b, ch) in trimmed.char_indices() {
                    if ch == ' ' {
                        if run_start.is_none() {
                            run_start = Some(b);
                        }
                    } else if let Some(rs) = run_start.take()
                        && b > rs + 1
                    {
                        spans.push(SpanContext::new(
                            input,
                            offset + rs,
                            offset + b,
                            depth,
                            parent_keys.clone(),
                        ));
                    }
                }
                if let Some(rs) = run_start.take()
                    && line_len > rs + 1
                {
                    spans.push(SpanContext::new(
                        input,
                        offset + rs,
                        offset + line_len,
                        depth,
                        parent_keys.clone(),
                    ));
                }
                if trimmed.trim().is_empty() {
                    empty_line_streak += 1;
                    if empty_line_streak >= 2 {
                        spans.push(SpanContext::new(
                            input,
                            offset,
                            offset + line.len(),
                            depth,
                            parent_keys.clone(),
                        ));
                    }
                } else {
                    empty_line_streak = 0;
                }
                offset += line.len();
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::ExcessWhitespaceOrNewline,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

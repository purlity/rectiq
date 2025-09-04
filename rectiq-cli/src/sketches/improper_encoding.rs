// rectiq-cli/src/sketches/improper_encoding.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

pub struct ImproperEncodingSketcher {
    maybe_has_u: bool,
}

impl Default for ImproperEncodingSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ImproperEncodingSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self { maybe_has_u: false }
    }
}

impl TokenSketcher for ImproperEncodingSketcher {
    fn name(&self) -> &'static str {
        "ImproperEncoding"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == 'u' {
            self.maybe_has_u = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_u {
            return None;
        }
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans = Vec::new();

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.kind != TokKind::StringLit {
                continue;
            }
            if lattice.class_for(tok.start) != RegionClass::String {
                continue;
            }
            if tok.end <= tok.start + 1 {
                continue;
            }
            let slice = &input[tok.start + 1..tok.end - 1];
            let mut rel = 0;
            while let Some(pos) = slice[rel..].find("\\u") {
                let esc_start = rel + pos;
                let hstart = esc_start + 2;
                if hstart + 4 <= slice.len() {
                    let hex = &slice[hstart..hstart + 4];
                    if hex.chars().all(|c| c.is_ascii_hexdigit())
                        && let Ok(v) = u16::from_str_radix(hex, 16)
                        && (0xD800..=0xDFFF).contains(&v)
                    {
                        let abs_start = tok.start + 1 + esc_start;
                        let abs_end = (abs_start + 6).min(tok.end);
                        let (parent_keys, depth) = skel.path_at(&tokens, input, idx);
                        spans.push(SpanContext::new(
                            input,
                            abs_start,
                            abs_end,
                            depth,
                            parent_keys,
                        ));
                    }
                }
                rel = hstart + 4;
                if rel >= slice.len() {
                    break;
                }
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::ImproperEncoding,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

// rectiq-cli/src/sketches/invalid_number_format.rs
use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect malformed numeric literals using the token stream. The lexer is
///
/// permissive, so we validate the textual representation of each `NumberLit`
/// token and flag those that do not conform to the JSON number grammar.
pub struct InvalidNumberFormatSketcher {
    maybe_has_digit: bool,
}

impl Default for InvalidNumberFormatSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl InvalidNumberFormatSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_digit: false,
        }
    }
}

impl TokenSketcher for InvalidNumberFormatSketcher {
    fn name(&self) -> &'static str {
        "InvalidNumberFormat"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c.is_ascii_digit() {
            self.maybe_has_digit = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_digit {
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
            if tok.kind == TokKind::NumberLit
                && !matches!(
                    lattice.class_for(tok.start),
                    RegionClass::Comment | RegionClass::String
                )
            {
                let start_idx = i;
                let mut end_idx = i + 1;
                let mut text = input[tok.start..tok.end].to_string();
                while end_idx < tokens.len()
                    && tokens[end_idx].kind == TokKind::NumberLit
                    && tokens[end_idx].start == tokens[end_idx - 1].end
                {
                    text.push_str(&input[tokens[end_idx].start..tokens[end_idx].end]);
                    end_idx += 1;
                }
                if !is_valid_number(&text) {
                    let (parent_keys, depth) = skel.path_at(&tokens, input, start_idx);
                    let span_start = tokens[start_idx].start;
                    let span_end = tokens[end_idx - 1].end;
                    spans.push(SpanContext::new(
                        input,
                        span_start,
                        span_end,
                        depth,
                        parent_keys,
                    ));
                }
                i = end_idx;
                continue;
            }
            i += 1;
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::InvalidNumberFormat,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

fn is_valid_number(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return false;
    }
    let mut i = 0;
    if bytes[i] == b'-' {
        i += 1;
        if i == len || !bytes[i].is_ascii_digit() {
            return false;
        }
    }
    if bytes[i] == b'0' {
        i += 1;
        if i < len && bytes[i].is_ascii_digit() {
            return false; // leading zero
        }
    } else {
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i < len && bytes[i] == b'.' {
        i += 1;
        let start = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if start == i {
            return false; // no digits after decimal
        }
    }
    if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < len && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        let start = i;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if start == i {
            return false; // missing exponent digits
        }
    }
    i == len
}

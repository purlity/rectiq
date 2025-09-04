// rectiq-cli/src/sketches/invalid_escape_sequence.rs
use crate::{TokenSketcher, pipeline::TokKind, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Detect invalid escape sequences within string literals using the lexer
///
/// token stream. Only operates on `TokKind::StringLit` tokens that are
/// classified as strings by the lattice.
pub struct InvalidEscapeSequenceSketcher {
    maybe_has_backslash: bool,
}

impl Default for InvalidEscapeSequenceSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl InvalidEscapeSequenceSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_backslash: false,
        }
    }
}

impl TokenSketcher for InvalidEscapeSequenceSketcher {
    fn name(&self) -> &'static str {
        "InvalidEscapeSequence"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == '\\' {
            self.maybe_has_backslash = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_backslash {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let input = &pool.input;
        let mut spans = Vec::new();

        for (idx, tok) in tokens.iter().enumerate() {
            if tok.kind != TokKind::StringLit {
                continue;
            }
            // Slice inside quotes
            if tok.end <= tok.start + 1 {
                continue;
            }
            let (parent_keys, depth) = skel.path_at(&tokens, input, idx);
            for (start, end) in collect_invalid_escapes(input, tok.start, tok.end) {
                spans.push(SpanContext::new(
                    input,
                    start,
                    end,
                    depth,
                    parent_keys.clone(),
                ));
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::InvalidEscapeSequence,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

fn collect_invalid_escapes(input: &str, span_start: usize, span_end: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if span_end <= span_start + 1 {
        return out;
    }
    let bytes = input.as_bytes();
    let mut i = span_start + 1; // skip opening quote
    while i < span_end - 1 {
        if bytes[i] == b'\\' {
            let esc_start = i;
            if i + 1 >= span_end - 1 {
                out.push((esc_start, (esc_start + 1).min(span_end)));
                break;
            }
            if bytes[i + 1] == b'\\' {
                if i + 2 < span_end - 1 {
                    let nx = bytes[i + 2];
                    if !matches!(
                        nx,
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' | b'u'
                    ) {
                        out.push((i + 1, (i + 3).min(span_end)));
                    }
                }
                i += 2;
                continue;
            }
            let next = bytes[i + 1];
            match next {
                b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                    i += 2;
                }
                b'u' => {
                    let mut ok = true;
                    let mut consumed = 0usize;
                    let mut j = i + 2;
                    for _ in 0..4 {
                        if j >= span_end - 1 {
                            ok = false;
                            break;
                        }
                        let d = bytes[j];
                        if d.is_ascii_hexdigit() {
                            j += 1;
                            consumed += 1;
                        } else {
                            ok = false;
                            j += 1;
                            consumed += 1;
                            break;
                        }
                    }
                    if !ok {
                        out.push((esc_start, (esc_start + 2 + consumed).min(span_end)));
                    }
                    i = j;
                }
                _ => {
                    out.push((esc_start, (i + 2).min(span_end)));
                    i += 2;
                }
            }
        } else {
            i += 1;
        }
    }
    out
}

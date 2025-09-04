// rectiq-cli/src/sketches/missing_quote.rs
use crate::{pipeline::TokKind, sketches::TokenSketcher, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Sketcher for missing quotes around object keys or string values.
pub struct MissingQuoteSketcher {
    maybe_has_colon: bool,
}

impl Default for MissingQuoteSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingQuoteSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_colon: false,
        }
    }
}

impl TokenSketcher for MissingQuoteSketcher {
    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn name(&self) -> &'static str {
        "MissingQuote"
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == ':' {
            self.maybe_has_colon = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_colon {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let _lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans = Vec::new();

        for (i, tok) in tokens.iter().enumerate() {
            if tok.kind != TokKind::Unknown {
                continue;
            }
            let Some(snippet) = input.get(tok.start..tok.end) else {
                continue;
            };
            if !snippet
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }

            // look ahead for ':' to detect unquoted key
            let mut j = i + 1;
            while j < tokens.len() && tokens[j].kind == TokKind::Whitespace {
                j += 1;
            }
            if j < tokens.len() && tokens[j].kind == TokKind::Colon {
                let (parent_keys, depth) = skel.path_at(&tokens, input, i);
                spans.push(SpanContext::new(
                    input,
                    tok.start,
                    tok.end,
                    depth,
                    parent_keys,
                ));
                continue;
            }

            // look behind for ':' to detect unquoted string value
            let mut k = i;
            while k > 0 {
                k -= 1;
                if tokens[k].kind == TokKind::Whitespace {
                    continue;
                }
                if tokens[k].kind == TokKind::Colon {
                    let (parent_keys, depth) = skel.path_at(&tokens, input, i);
                    spans.push(SpanContext::new(
                        input,
                        tok.start,
                        tok.end,
                        depth,
                        parent_keys,
                    ));
                }
                break;
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::MissingQuote,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

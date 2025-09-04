// rectiq-cli/src/sketches/missing_comma_between_item.rs
use crate::{pipeline::TokKind, sketches::TokenSketcher, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

/// Sketcher for detecting missing commas between items in arrays or objects.
pub struct MissingCommaBetweenItemSketcher {
    maybe_has_colon_or_value: bool,
}

impl Default for MissingCommaBetweenItemSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MissingCommaBetweenItemSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_colon_or_value: false,
        }
    }
}

impl TokenSketcher for MissingCommaBetweenItemSketcher {
    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn name(&self) -> &'static str {
        "MissingCommaBetweenItem"
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == ':' || c.is_ascii_alphanumeric() {
            self.maybe_has_colon_or_value = true;
        }
    }

    #[allow(clippy::too_many_lines)]
    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_colon_or_value {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let input = &pool.input;

        let mut spans = Vec::new();
        for (i, tok) in tokens.iter().enumerate() {
            // Only consider value starts and composite starts as potential "current" values
            let is_value_start = matches!(
                tok.kind,
                TokKind::StringLit
                    | TokKind::NumberLit
                    | TokKind::True
                    | TokKind::False
                    | TokKind::Null
                    | TokKind::LBrace
                    | TokKind::LBracket
            );
            if !is_value_start {
                continue;
            }

            // Find previous significant token (skip whitespace/comments)
            let mut p = i;
            while p > 0 {
                p -= 1;
                if !matches!(tokens[p].kind, TokKind::Whitespace | TokKind::Comment) {
                    break;
                }
            }
            if p == i {
                continue; // no previous token
            }

            // If there is a comma or colon between previous and current, it's fine
            let mut k = p + 1;
            let mut has_sep = false;
            while k < i {
                if matches!(tokens[k].kind, TokKind::Comma | TokKind::Colon) {
                    has_sep = true;
                    break;
                }
                k += 1;
            }
            if has_sep {
                continue;
            }

            // The previous significant token must end a value (literal or composite close)
            if !matches!(
                tokens[p].kind,
                TokKind::StringLit
                    | TokKind::NumberLit
                    | TokKind::True
                    | TokKind::False
                    | TokKind::Null
                    | TokKind::RBrace
                    | TokKind::RBracket
            ) {
                continue;
            }

            // Same-parent sanity: require same depth + path
            let (parent_keys_cur, depth_cur) = skel.path_at(&tokens, input, i);
            let (parent_keys_prev, depth_prev) = skel.path_at(&tokens, input, p);
            if depth_cur != depth_prev || parent_keys_cur != parent_keys_prev {
                continue;
            }

            // Emit the GAP between values for clarity
            let left_end = tokens[p].end;
            let right_start = tok.start;
            let (start, end) = if right_start > left_end {
                (left_end, right_start)
            } else {
                (tok.start, tok.start.saturating_add(1))
            };
            spans.push(SpanContext::new(
                input,
                start,
                end,
                depth_cur,
                parent_keys_cur,
            ));
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::MissingCommaBetweenItem,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

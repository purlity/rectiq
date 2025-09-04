// rectiq-cli/src/sketches/empty_key_or_value.rs
use crate::{TokenSketcher, pipeline::TokKind, types::pool::LocalShapePool};
use rectiq_types::{
    Kind, SketchNode, SketchPayload, SpanContext, span_utils::merge_adjacent_single_char_spans,
};

pub struct EmptyKeyOrValueSketcher {
    maybe_has_colon: bool,
    maybe_has_quote: bool,
}

impl Default for EmptyKeyOrValueSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyKeyOrValueSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_colon: false,
            maybe_has_quote: false,
        }
    }
}

impl TokenSketcher for EmptyKeyOrValueSketcher {
    fn name(&self) -> &'static str {
        "EmptyKeyOrValue"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self {
            maybe_has_colon: self.maybe_has_colon,
            maybe_has_quote: self.maybe_has_quote,
        })
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == ':' {
            self.maybe_has_colon = true;
        }
        if c == '"' {
            self.maybe_has_quote = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_colon && !self.maybe_has_quote {
            return None;
        }
        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let _lattice = pool.pipeline_lattice();
        let input = &pool.input;

        let mut spans = Vec::new();

        for pair in &skel.obj_pairs {
            let key_idx = pair.key_span.0;
            let key_tok = &tokens[key_idx];
            if key_tok.end == key_tok.start + 2 {
                let (parent_keys, depth) = skel.path_at(&tokens, input, key_idx);
                spans.push(SpanContext::new(
                    input,
                    key_tok.start,
                    key_tok.end,
                    depth,
                    parent_keys,
                ));
            }
            let val_idx = pair.value_span.0;
            let val_tok = &tokens[val_idx];
            if val_tok.kind == TokKind::StringLit && val_tok.end == val_tok.start + 2 {
                let (parent_keys, depth) = skel.path_at(&tokens, input, val_idx);
                spans.push(SpanContext::new(
                    input,
                    val_tok.start,
                    val_tok.end,
                    depth,
                    parent_keys,
                ));
            }
        }

        // missing values
        for (i, tok) in tokens.iter().enumerate() {
            if tok.kind != TokKind::Colon {
                continue;
            }
            let mut j = i;
            while j > 0 && matches!(tokens[j - 1].kind, TokKind::Whitespace | TokKind::Comment) {
                j -= 1;
            }
            if j == 0 {
                continue;
            }
            let key_tok = &tokens[j - 1];
            if key_tok.kind != TokKind::StringLit {
                continue;
            }
            let mut k = i + 1;
            while k < tokens.len()
                && matches!(tokens[k].kind, TokKind::Whitespace | TokKind::Comment)
            {
                k += 1;
            }
            if k >= tokens.len() {
                continue;
            }
            if matches!(
                tokens[k].kind,
                TokKind::Comma | TokKind::RBrace | TokKind::RBracket
            ) {
                let key_text = {
                    let s = key_tok.start + 1;
                    let e = key_tok.end.saturating_sub(1);
                    if e >= s && e <= input.len() {
                        &input[s..e]
                    } else {
                        ""
                    }
                };
                let (mut parent_keys, depth) = skel.path_at(&tokens, input, i);
                parent_keys.push(std::borrow::Cow::Owned(key_text.to_string()));
                spans.push(SpanContext::new(
                    input,
                    tok.start,
                    tokens[k].end,
                    depth + 1,
                    parent_keys,
                ));
            }
        }

        if spans.is_empty() {
            None
        } else {
            let merged = merge_adjacent_single_char_spans(input, spans);
            Some(SketchNode {
                kind: Kind::EmptyKeyOrValue,
                fix_hint: None,
                payload: SketchPayload::Spans(merged),
            })
        }
    }
}

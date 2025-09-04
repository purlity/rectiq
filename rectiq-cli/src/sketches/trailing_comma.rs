// rectiq-cli/src/sketches/trailing_comma.rs

use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{Kind, SketchNode, SketchPayload, SpanContext};

/// Detects commas that appear immediately before a closing `]` or `}`.
///
/// The detector operates purely on the lexer's token stream and the structural
/// skeleton. Only commas that reside within arrays or objects are considered,
/// and spans inside comments or strings are ignored via the lattice
/// classification.
pub struct TrailingCommaSketcher {
    maybe_has_comma: bool,
}

impl Default for TrailingCommaSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl TrailingCommaSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_comma: false,
        }
    }
}

impl TokenSketcher for TrailingCommaSketcher {
    fn name(&self) -> &'static str {
        "TrailingComma"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == ',' {
            self.maybe_has_comma = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_comma {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let lattice = pool.pipeline_lattice();
        let mut spans = Vec::new();
        let input = &pool.input;

        for (i, tok) in tokens.iter().enumerate() {
            if !matches!(tok.kind, TokKind::RBracket | TokKind::RBrace) {
                continue;
            }

            // find preceding non-whitespace/comment token
            let mut j = i;
            while j > 0 {
                j -= 1;
                match tokens[j].kind {
                    TokKind::Whitespace | TokKind::Comment => {}
                    _ => break,
                }
            }

            if tokens[j].kind != TokKind::Comma {
                continue;
            }
            // Skip spans that fall inside comments/strings
            if matches!(
                lattice.class_for(tokens[j].start),
                RegionClass::Comment | RegionClass::String
            ) {
                continue;
            }

            let (parent_keys, depth) = skel.path_at(&tokens, input, j);
            spans.push(SpanContext::new(
                input,
                tokens[j].start,
                tokens[j].end,
                depth,
                parent_keys,
            ));
        }

        if spans.is_empty() {
            None
        } else {
            Some(SketchNode {
                kind: Kind::TrailingComma,
                fix_hint: None,
                payload: SketchPayload::Spans(spans),
            })
        }
    }
}

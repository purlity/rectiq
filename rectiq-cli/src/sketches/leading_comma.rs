// rectiq-cli/src/sketches/leading_comma.rs

use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{Kind, SketchNode, SketchPayload, SpanContext};

/// Detects commas that appear immediately after an opening `[` or `{`.
///
/// The detector works purely on the SUPRA pipeline tokens and ignores commas
/// that occur inside comments or strings.
pub struct LeadingCommaSketcher {
    maybe_has_comma: bool,
}

impl Default for LeadingCommaSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl LeadingCommaSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_comma: false,
        }
    }
}

impl TokenSketcher for LeadingCommaSketcher {
    fn name(&self) -> &'static str {
        "LeadingComma"
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
            if !matches!(tok.kind, TokKind::LBracket | TokKind::LBrace) {
                continue;
            }
            let mut j = i + 1;
            while j < tokens.len()
                && matches!(tokens[j].kind, TokKind::Whitespace | TokKind::Comment)
            {
                j += 1;
            }
            if j >= tokens.len() || tokens[j].kind != TokKind::Comma {
                continue;
            }
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
                kind: Kind::LeadingComma,
                fix_hint: None,
                payload: SketchPayload::Spans(spans),
            })
        }
    }
}

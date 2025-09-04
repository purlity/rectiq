// rectiq-cli/src/sketches/double_comma.rs

use crate::{
    TokenSketcher,
    pipeline::{RegionClass, TokKind},
    types::pool::LocalShapePool,
};
use rectiq_types::{Kind, SketchNode, SketchPayload, SpanContext};

/// Detects multiple commas appearing consecutively between array or object
///
/// values. The commas themselves (including any run of them) are reported as a
/// single span. Spans inside comments or strings are ignored via lattice
/// classification.
pub struct DoubleCommaSketcher {
    maybe_has_comma: bool,
}

impl Default for DoubleCommaSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DoubleCommaSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_comma: false,
        }
    }
}

impl TokenSketcher for DoubleCommaSketcher {
    fn name(&self) -> &'static str {
        "DoubleComma"
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
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].kind == TokKind::Comma {
                let mut j = i + 1;
                while j < tokens.len()
                    && matches!(tokens[j].kind, TokKind::Whitespace | TokKind::Comment)
                {
                    j += 1;
                }
                if j < tokens.len() && tokens[j].kind == TokKind::Comma {
                    let mut last = j;
                    let mut k = j + 1;
                    while k < tokens.len() {
                        while k < tokens.len()
                            && matches!(tokens[k].kind, TokKind::Whitespace | TokKind::Comment)
                        {
                            k += 1;
                        }
                        if k < tokens.len() && tokens[k].kind == TokKind::Comma {
                            last = k;
                            k += 1;
                        } else {
                            break;
                        }
                    }
                    let start = tokens[i].start;
                    let end = tokens[last].end;
                    if !matches!(
                        lattice.class_for(start),
                        RegionClass::Comment | RegionClass::String
                    ) {
                        let (parent_keys, depth) = skel.path_at(&tokens, input, i);
                        spans.push(SpanContext::new(input, start, end, depth, parent_keys));
                    }
                    i = last + 1;
                    continue;
                }
            }
            i += 1;
        }

        if spans.is_empty() {
            None
        } else {
            Some(SketchNode {
                kind: Kind::DoubleComma,
                fix_hint: None,
                payload: SketchPayload::Spans(spans),
            })
        }
    }
}

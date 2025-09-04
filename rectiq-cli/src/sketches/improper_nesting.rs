// rectiq-cli/src/sketches/improper_nesting.rs
use crate::{TokenSketcher, pipeline::RegionClass, types::pool::LocalShapePool};
use rectiq_types::{Kind, SketchNode, SketchPayload, SpanContext};

pub struct ImproperNestingSketcher;

impl Default for ImproperNestingSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ImproperNestingSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TokenSketcher for ImproperNestingSketcher {
    fn name(&self) -> &'static str {
        "ImproperNesting"
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

        let mut spans = Vec::new();
        for byte in &skel.bracket_mismatches {
            if let Some((idx, tok)) = tokens.iter().enumerate().find(|(_, t)| t.start == *byte) {
                if matches!(
                    lattice.class_for(tok.start),
                    RegionClass::Comment | RegionClass::String
                ) {
                    continue;
                }
                let (parent_keys, depth) = skel.path_at(&tokens, input, idx);
                spans.push(SpanContext::new(
                    input,
                    tok.start,
                    tok.end,
                    depth,
                    parent_keys,
                ));
            }
        }

        if spans.is_empty() {
            None
        } else {
            Some(SketchNode {
                kind: Kind::ImproperNesting,
                fix_hint: None,
                payload: SketchPayload::Spans(spans),
            })
        }
    }
}

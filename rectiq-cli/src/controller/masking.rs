// rectiq-cli/src/controller/masking.rs (new)
use crate::utils::crypto::{mask, mask_json_path, mask_pointer_str};
use rectiq_types::{FixHint, KeyPair, RefEdge, SketchNode, SketchPayload, SpanContext};
use std::borrow::Cow;

#[must_use]
pub fn to_masked_owned_envelope(node: &SketchNode) -> SketchNode<'static> {
    let kind = *node.kind();
    let fix_hint = node.fix_hint.as_ref().map(FixHint::clone_static);

    let payload = match &node.payload {
        SketchPayload::Spans(spans) => {
            let masked: Vec<SpanContext<'static>> = spans
                .iter()
                .map(|s| {
                    let mut owned = s.clone_owned();
                    owned.span.snippet = owned
                        .span
                        .snippet
                        .as_deref()
                        .map(|t| Cow::Owned(mask(t).into_owned()));
                    owned.parent_keys = owned
                        .parent_keys
                        .into_iter()
                        .map(|k| Cow::Owned(mask(&k).into_owned()))
                        .collect();
                    owned
                })
                .collect();
            SketchPayload::Spans(masked)
        }
        SketchPayload::Pairs(pairs) => {
            let masked: Vec<KeyPair<'static>> = pairs
                .iter()
                .map(|kp| {
                    let mut owned = kp.clone_owned();
                    owned.key = Cow::Owned(mask(&owned.key).into_owned());
                    owned.key_id = Cow::Owned(mask(&owned.key_id).into_owned());
                    if let Some(s) = owned.snippet.as_deref() {
                        owned.snippet = Some(Cow::Owned(mask(s).into_owned()));
                    }
                    owned.parent_keys = mask_json_path(&owned.parent_keys);
                    owned
                })
                .collect();
            SketchPayload::Pairs(masked)
        }
        SketchPayload::Edges(edges) => {
            let masked: Vec<RefEdge<'static>> = edges
                .iter()
                .map(|e| {
                    let mut owned = e.clone_owned();
                    owned.from = mask_json_path(&owned.from);
                    owned.to = mask_json_path(&owned.to);
                    if let Some(ptr) = owned.to_pointer.as_deref() {
                        owned.to_pointer = Some(Cow::Owned(mask_pointer_str(ptr)));
                    }
                    if let Some(s) = owned.snippet.as_deref() {
                        owned.snippet = Some(Cow::Owned(mask(s).into_owned()));
                    }
                    owned
                })
                .collect();
            SketchPayload::Edges(masked)
        }
    };

    SketchNode {
        kind,
        payload,
        fix_hint,
    }
}

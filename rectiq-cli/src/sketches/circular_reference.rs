// rectiq-cli/src/sketches/circular_reference.rs
use crate::{
    pipeline::TokKind,
    sketches::{TokenSketcher, circular::detect_ref_cycles},
    types::pool::LocalShapePool,
};
use rectiq_types::{ByteSpan, JsonPath, JsonPathSegment, Kind, RefEdge, SketchNode, SketchPayload};
use std::borrow::Cow;

/// Build a `JsonPath` from a slice of parent key segments (all treated as string segments).
fn json_path_from_keys<'a>(keys: &[Cow<'a, str>]) -> JsonPath<'a> {
    JsonPath(keys.iter().cloned().map(JsonPathSegment::Str).collect())
}

/// Convert a JSON Pointer string (e.g., "#/a/b/0") into a `JsonPath`.
/// Assumes it starts with '#'. Decodes RFC6901 escapes: ~1 -> '/', ~0 -> '~'.
fn pointer_to_json_path(ptr: &str) -> JsonPath<'static> {
    let no_hash = ptr.strip_prefix('#').unwrap_or(ptr);
    let mut segments: Vec<JsonPathSegment<'static>> = Vec::new();
    // allow both pointer form (starts with '/') or dotted fallback converted earlier
    for raw in no_hash.split('/') {
        if raw.is_empty() {
            continue;
        }
        // decode ~1 and ~0
        let decoded = raw.replace("~1", "/").replace("~0", "~");
        // try index
        if let Ok(idx) = decoded.parse::<usize>() {
            segments.push(JsonPathSegment::Index(idx));
        } else {
            segments.push(JsonPathSegment::Str(Cow::Owned(decoded)));
        }
    }
    JsonPath(segments)
}

/// Streaming Sketcher for circular references.
pub struct CircularReferenceSketcher {
    maybe_has_dollar_ref: bool,
}

impl Default for CircularReferenceSketcher {
    fn default() -> Self {
        Self::new()
    }
}

impl CircularReferenceSketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_has_dollar_ref: false,
        }
    }
}

impl TokenSketcher for CircularReferenceSketcher {
    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self::new())
    }

    fn name(&self) -> &'static str {
        "CircularReference"
    }

    fn observe(&mut self, c: char, _offset: usize) {
        if c == '$' {
            self.maybe_has_dollar_ref = true;
        }
    }

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>> {
        if !self.maybe_has_dollar_ref {
            return None;
        }

        let tokens = pool.pipeline_tokens();
        let skel = pool.pipeline_skeleton();
        let input = &pool.input;

        // Collect raw $ref edges from tokens/skeleton
        let mut edges: Vec<RefEdge<'a>> = Vec::new();
        for pair in &skel.obj_pairs {
            let key_tok_idx = pair.key_span.0;
            let key_tok = &tokens[key_tok_idx];
            if key_tok.kind != TokKind::StringLit || key_tok.end <= key_tok.start + 1 {
                continue;
            }
            let key_snip = &input[key_tok.start + 1..key_tok.end - 1];
            if key_snip != "$ref" {
                continue;
            }

            // Value must be a string literal beginning with '#'
            let val_tok_idx = pair.value_span.0;
            let val_tok = &tokens[val_tok_idx];
            if val_tok.kind != TokKind::StringLit || val_tok.end <= val_tok.start + 1 {
                continue;
            }
            let value_snip = &input[val_tok.start + 1..val_tok.end - 1];
            if !value_snip.starts_with('#') {
                continue;
            }

            // Normalize a dotted path (e.g. "#a.b.0") into a JSON Pointer ("#/a/b/0").
            // If the string already uses pointer form (starts with "#/") we keep it.
            let to_pointer = if value_snip.starts_with("#/") {
                value_snip.to_string()
            } else {
                let mut out = String::from("#");
                for seg in value_snip.trim_start_matches('#').split('.') {
                    if seg.is_empty() {
                        continue;
                    }
                    out.push('/');
                    out.push_str(seg);
                }
                out
            };

            // Byte span for the reference value content inside quotes
            let span = Some(ByteSpan {
                start: val_tok.start + 1,
                end: val_tok.end - 1,
            });

            // Build context info from skeleton at the value token index
            let (parent_keys, depth) = skel.path_at(&tokens, input, val_tok_idx);
            let from_base = json_path_from_keys(&parent_keys);
            // from path includes the key name "$ref" under the current object
            let mut from_vec = from_base.0.clone();
            from_vec.push(JsonPathSegment::Str(Cow::Borrowed("$ref")));
            let from_path = JsonPath(from_vec);

            // to path from pointer
            let to_path = pointer_to_json_path(&to_pointer);

            edges.push(RefEdge {
                from: from_path,
                to: to_path,
                span,
                context_depth: depth,
                parent_keys: from_base,
                to_pointer: Some(Cow::Owned(to_pointer)),
                snippet: Some(Cow::Borrowed(value_snip)),
            });
        }

        // Detect cycles among the gathered edges, then include only those in cycles
        let mut cyclical: Vec<RefEdge> = Vec::new();
        for cycle in detect_ref_cycles(&edges) {
            for e in cycle {
                cyclical.push(e.clone());
            }
        }

        if cyclical.is_empty() {
            None
        } else {
            Some(SketchNode {
                kind: Kind::CircularReference,
                fix_hint: None,
                payload: SketchPayload::Edges(cyclical),
            })
        }
    }
}

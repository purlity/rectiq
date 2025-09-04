// rectiq-cli/src/remote/helper.rs
use rectiq_types::{JsonPathSegment, SketchNode, SketchPayload};
use std::collections::HashSet;

pub fn insert_chars<S: ::std::hash::BuildHasher>(s: &str, set: &mut HashSet<char, S>) {
    for c in s.chars() {
        if !c.is_whitespace() {
            set.insert(c);
        }
    }
}

pub fn collect_chars_from_sketch<S: ::std::hash::BuildHasher>(
    sketch: &SketchNode<'_>,
    set: &mut HashSet<char, S>,
) {
    match &sketch.payload {
        SketchPayload::Spans(span_refs) => {
            insert_chars(sketch.kind.as_str(), set);
            for ctx in span_refs {
                insert_chars(&ctx.start().to_string(), set);
                insert_chars(&ctx.end().to_string(), set);
                if let Some(sn) = ctx.snippet() {
                    insert_chars(sn, set);
                }
            }
        }
        SketchPayload::Pairs(key_pairs) => {
            insert_chars(sketch.kind.as_str(), set);
            for br in key_pairs {
                insert_chars(&br.pair_span.start.to_string(), set);
                insert_chars(&br.pair_span.end.to_string(), set);
                insert_chars(&br.key_span.start.to_string(), set);
                insert_chars(&br.key_span.end.to_string(), set);
                for key in &br.parent_keys.0 {
                    insert_chars(&key.to_string(), set);
                }
                insert_chars(&br.context_depth.to_string(), set);
                insert_chars(&br.key, set);
                insert_chars(&br.key_id, set);
                if let Some(sn) = &br.snippet {
                    insert_chars(sn.as_ref(), set);
                }
            }
        }
        SketchPayload::Edges(ref_edges) => {
            insert_chars(sketch.kind.as_str(), set);
            for e in ref_edges {
                if let Some(span) = e.span {
                    insert_chars(&span.start.to_string(), set);
                    insert_chars(&span.end.to_string(), set);
                } else if let Some(s) = e.start() {
                    insert_chars(&s.to_string(), set);
                }
                insert_chars(&e.context_depth.to_string(), set);
                for key in e.parent_keys.iter() {
                    match key {
                        JsonPathSegment::Str(s) => insert_chars(s.as_ref(), set),
                        JsonPathSegment::Index(i) => insert_chars(&i.to_string(), set),
                    }
                }
                if let Some(ptr) = &e.to_pointer {
                    insert_chars(ptr.as_ref(), set);
                }
                if let Some(sn) = &e.snippet {
                    insert_chars(sn.as_ref(), set);
                }
            }
        }
    }
    // 👇 Defensive fallback: serialize the entire sketch to JSON string & extract missed chars
    if let Ok(serialized) = serde_json::to_string(sketch) {
        insert_chars(&serialized, set);
    }
}

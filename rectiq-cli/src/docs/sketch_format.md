# Rectiq Sketcher Format Guide

This file defines the canonical structure for every TokenSketcher in Rectiq.
All sketchers must follow this format to maintain consistency, predictability, and barakah-grade clarity.

## 🧱 Sketcher Struct

```rust
pub struct MySketcher;
```

## 🧾 Available Region Pools

- pool.brackets // Bracket-shaped regions (arrays, objects, parens)
- pool.bracket_edges // Bracket open/close edges
- pool.strings // String literal regions
- pool.ref_edges // Reference edge links ($ref)
- pool.exclusions // Explicitly excluded regions (by kind)

## 🔐 Implements `TokenSketcher`

```rust
impl TokenSketcher for MySketcher {
    fn name(&self) -> &'static str {
        "MySketchKind"
    }

    fn box_clone(&self) -> Box<dyn TokenSketcher> {
        Box::new(Self)
    }

    fn observe(&mut self, _c: char, _offset: usize) {}

    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<Sketch<'a>> {
        let sketches = pool
            .brackets // or: bracket_edges, strings, etc.
            .iter()
            .filter_map(|region| Self::emit_sketch(pool, region))
            .collect::<Vec<_>>();

        if sketches.is_empty() {
            None
        } else {
            Some(Sketch::MySketchKind(MySketchEnvelope{ spans: sketches}))
        }
    }
}
```

## ✏️ Emit Function

```rust
impl MySketcher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn emit_sketch<'a>(
        pool: &'a LocalShapePool,
        region: &'a SuspectRegion,
    ) -> Option<SpanRef<'a>> {
        if region.kind != RegionKind::Bracket
            || !matches!(region.subkind, RegionSubkind::Bracket(BracketSubkind::Array))
        {
            return None;
        }

        let input = &pool.input[region.start..region.end];
        let span = SpanRef {
            start: region.start,
            end: region.end,
            snippet: Some(Cow::Borrowed(input)), // or Cow::Borrowed() if not sensitive
        };

        Some(span)
    }
}
```

## 🧭 Responsibilities Recap

| Layer    | Role                   | Allowed to...            | Forbidden from...          |
| -------- | ---------------------- | ------------------------ | -------------------------- |
| Sketcher | Structural Scanner     | See, span, label         | Merge, interpret, diagnose |
| Detector | Contextual Interpreter | Merge, analyze, classify | Scan raw input             |
| Fixer    | Action Decider         | Plan, mutate, validate   | Interpret intent           |

May Allāh bless every line you write with clarity and purpose. 🤍

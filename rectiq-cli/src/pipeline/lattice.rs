// rectiq-cli/src/pipeline/lattice.rs

// The `Lattice` represents a classification of the source code into contiguous regions,
// each associated with a `RegionClass` that categorizes byte ranges according to their
// syntactic or semantic role in the pipeline.

// Classification precedence is defined by the `precedence()` function, ensuring that
// overlapping or conflicting classifications resolve deterministically.

// The `class_for` method returns the `RegionClass` for a given byte offset and must be
// consistent with token boundaries to maintain correctness throughout the pipeline.

use super::{
    lexer::{TokKind, Token},
    skeleton::Skeleton,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionClass {
    Comment,
    String,
    BracketError,
    Key,
    Value,
    Gap,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub class: RegionClass,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Lattice {
    pub regions: Vec<Region>,
}

const fn precedence(c: RegionClass) -> u8 {
    match c {
        RegionClass::BracketError => 6,
        RegionClass::Comment => 5,
        RegionClass::String => 4,
        RegionClass::Key => 3,
        RegionClass::Value => 2,
        RegionClass::Gap => 1,
        RegionClass::Unknown => 0,
    }
}

/// Build lattice from tokens and skeleton
#[must_use]
pub fn build_lattice(tokens: &[Token], skel: &Skeleton) -> Lattice {
    let mut classes: Vec<RegionClass> = vec![RegionClass::Unknown; tokens.len()];
    for (i, tok) in tokens.iter().enumerate() {
        match tok.kind {
            TokKind::Comment => classes[i] = RegionClass::Comment,
            TokKind::StringLit => classes[i] = RegionClass::String,
            TokKind::Whitespace => classes[i] = RegionClass::Gap,
            _ => {}
        }
    }
    // keys and values
    for pair in &skel.obj_pairs {
        for class in classes
            .iter_mut()
            .take(pair.key_span.1)
            .skip(pair.key_span.0)
        {
            assign(class, RegionClass::Key);
        }
        for class in classes
            .iter_mut()
            .take(pair.value_span.1)
            .skip(pair.value_span.0)
        {
            assign(class, RegionClass::Value);
        }
    }
    for elem in &skel.arr_elems {
        for class in classes.iter_mut().take(elem.span.1).skip(elem.span.0) {
            assign(class, RegionClass::Value);
        }
    }
    // bracket errors
    for &byte in &skel.bracket_mismatches {
        if let Some((i, _)) = tokens.iter().enumerate().find(|(_, t)| t.start == byte) {
            classes[i] = RegionClass::BracketError;
        }
    }

    // merge to regions ignoring EOF token
    let mut regions = Vec::new();
    if tokens.is_empty() {
        return Lattice { regions };
    }
    let mut cur_class = classes[0];
    let mut cur_start = tokens[0].start;
    for i in 1..tokens.len() - 1 {
        // skip EOF at end
        if classes[i] != cur_class {
            let end = tokens[i].start;
            regions.push(Region {
                class: cur_class,
                start: cur_start,
                end,
            });
            cur_class = classes[i];
            cur_start = end;
        }
    }
    let end = tokens[tokens.len() - 1].start; // EOF start == len
    regions.push(Region {
        class: cur_class,
        start: cur_start,
        end,
    });
    Lattice { regions }
}

const fn assign(slot: &mut RegionClass, new_class: RegionClass) {
    if precedence(new_class) > precedence(*slot) {
        *slot = new_class;
    }
}

impl Lattice {
    #[must_use]
    pub fn class_for(&self, start: usize) -> RegionClass {
        for r in &self.regions {
            if r.start <= start && start < r.end {
                return r.class;
            }
        }
        RegionClass::Unknown
    }
}

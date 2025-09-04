// rectiq-cli/src/pipeline/mod.rs
//
// Unified token ➜ skeleton ➜ lattice pipeline.
// Detectors consume these structured views only; raw bytes are off limits.

// Invariants:
// - Lexer produces full coverage Token stream over [0..len], plus Eof at [len..len].
// - Skeleton identifies bracket structure, obj key/value, array elems, and mismatch culprits.
// - Lattice emits disjoint, ordered regions covering [0..len] with precedence:
//   BracketError > Comment > String > Key > Value > Gap > Unknown.
pub mod lattice;
pub mod lexer;
pub mod skeleton;

pub use lattice::{Lattice, Region, RegionClass, build_lattice};
pub use lexer::{TokKind, Token, lex};
pub use skeleton::{Skeleton, build_skeleton};

pub struct Scan<T> {
    pub input: String,
    pub inner: T,
}

pub struct Tokens {
    pub tokens: Vec<Token>,
}
pub struct Skeletonized {
    pub tokens: Vec<Token>,
    pub skel: Skeleton<'static>,
}
pub struct Latticed {
    pub tokens: Vec<Token>,
    pub skel: Skeleton<'static>,
    pub lattice: Lattice,
}

impl Scan<()> {
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            inner: (),
        }
    }
    #[must_use]
    pub fn lex(self) -> Scan<Tokens> {
        let tokens = lex(&self.input);
        Scan {
            input: self.input,
            inner: Tokens { tokens },
        }
    }
}

impl Scan<Tokens> {
    #[must_use]
    pub fn build_skeleton(self) -> Scan<Skeletonized> {
        let skel = build_skeleton(&self.input, &self.inner.tokens);
        Scan {
            input: self.input,
            inner: Skeletonized {
                tokens: self.inner.tokens,
                skel,
            },
        }
    }
}

impl Scan<Skeletonized> {
    #[must_use]
    pub fn build_lattice(self) -> Scan<Latticed> {
        let lattice = build_lattice(&self.inner.tokens, &self.inner.skel);
        Scan {
            input: self.input,
            inner: Latticed {
                tokens: self.inner.tokens,
                skel: self.inner.skel,
                lattice,
            },
        }
    }
}

impl Latticed {
    #[inline]
    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    #[inline]
    #[must_use]
    pub fn class_for(&self, byte: usize) -> RegionClass {
        self.lattice.class_for(byte)
    }
}

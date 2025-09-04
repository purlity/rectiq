// rectiq-cli/src/types/pool.rs
use crate::pipeline::{self, Lattice, RegionClass, Skeleton, Token};
use std::cell::RefCell;
/// ---
///
/// 💡 ZK Compliance Note:
///
/// `LocalShapePool` owns the full input **only inside rectiq-cli**.
/// It never crosses the client boundary.
///
/// With the SUPRA pipeline (lexer → skeleton → lattice),
/// the pool lazily builds and caches the substrate for all sketchers.
/// No legacy scanner caches remain.
///
/// ---

#[derive(Default)]
pub struct LocalShapePool {
    pub input: String,
    pub pipeline_tokens: RefCell<Option<Vec<Token>>>,
    pub pipeline_skeleton: RefCell<Option<Skeleton<'static>>>,
    pub pipeline_lattice: RefCell<Option<Lattice>>,
}

impl LocalShapePool {
    #[must_use]
    pub fn new(input: &str) -> Self {
        Self {
            input: input.to_string(),
            pipeline_tokens: RefCell::new(None),
            pipeline_skeleton: RefCell::new(None),
            pipeline_lattice: RefCell::new(None),
        }
    }
}

impl LocalShapePool {
    /// Build the SUPRA scanning pipeline once and cache results.
    fn ensure_pipeline(&self) {
        if self.pipeline_tokens.borrow().is_some() {
            return;
        }
        let scan = pipeline::Scan::new(&self.input)
            .lex()
            .build_skeleton()
            .build_lattice();
        // Move owned results into the cache
        *self.pipeline_tokens.borrow_mut() = Some(scan.inner.tokens);
        *self.pipeline_skeleton.borrow_mut() = Some(scan.inner.skel);
        *self.pipeline_lattice.borrow_mut() = Some(scan.inner.lattice);
    }

    /// Accessors for the cached pipeline. These take &self and return clones of the data.
    #[must_use]
    /// # Panics
    /// Panics if the pipeline tokens are not cached.
    pub fn pipeline_tokens(&self) -> Vec<Token> {
        self.ensure_pipeline();
        self.pipeline_tokens
            .borrow()
            .as_ref()
            .expect("pipeline tokens cached")
            .clone()
    }

    #[must_use]
    /// # Panics
    /// Panics if the pipeline skeleton is not cached.
    pub fn pipeline_skeleton(&self) -> Skeleton<'static> {
        self.ensure_pipeline();
        self.pipeline_skeleton
            .borrow()
            .as_ref()
            .expect("pipeline skeleton cached")
            .clone()
    }

    #[must_use]
    /// # Panics
    /// Panics if the pipeline lattice is not cached.
    pub fn pipeline_lattice(&self) -> Lattice {
        self.ensure_pipeline();
        self.pipeline_lattice
            .borrow()
            .as_ref()
            .expect("pipeline lattice cached")
            .clone()
    }

    /// Convenience passthrough: region classification at a byte index.
    #[must_use]
    /// # Panics
    /// Panics if the pipeline lattice is not cached.
    pub fn class_for(&self, byte: usize) -> RegionClass {
        self.ensure_pipeline();
        self.pipeline_lattice
            .borrow()
            .as_ref()
            .expect("pipeline lattice cached")
            .class_for(byte)
    }
}

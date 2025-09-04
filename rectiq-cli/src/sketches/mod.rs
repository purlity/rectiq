// rectiq-client/src/sketches/mod.rs

// Orchestrates all sketchers over the SUPRA pipeline.
// Deduplicates spans globally and sorts results by `Kind::priority()`.
use crate::types::{pool::LocalShapePool, scan::LocalScan};
use rectiq_types::span_utils::dedup_spans;
use rectiq_types::{SketchNode, SketchPayload};
use std::panic::UnwindSafe;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;
use tracing::{info, info_span, instrument};

pub mod circular;
pub mod circular_reference;
pub mod comment_in_json;
pub mod double_comma;
pub mod duplicate_key;
pub mod empty_key_or_value;
pub mod excess_whitespace_or_newline;
pub mod extra_or_missing_bracket_complex;
pub mod extra_or_missing_colon;
pub mod improper_encoding;
pub mod improper_nesting;
pub mod incorrect_boolean_literal;
pub mod invalid_character;
pub mod invalid_escape_sequence;
pub mod invalid_number_format;
pub mod leading_comma;
pub mod missing_comma_between_item;
pub mod missing_quote;
pub mod mixed_type_in_array;
pub mod null_or_none_literal;
pub mod overly_large_number;
pub mod trailing_comma;
pub mod unbalanced_bracket;
pub mod unescaped_quote;
pub mod unexpected_token;

pub use circular_reference::CircularReferenceSketcher;
pub use comment_in_json::CommentInJsonSketcher;
pub use double_comma::DoubleCommaSketcher;
pub use duplicate_key::DuplicateKeySketcher;
pub use empty_key_or_value::EmptyKeyOrValueSketcher;
pub use excess_whitespace_or_newline::ExcessWhitespaceOrNewlineSketcher;
pub use extra_or_missing_bracket_complex::ExtraOrMissingBracketComplexSketcher;
pub use extra_or_missing_colon::ExtraOrMissingColonSketcher;
pub use improper_encoding::ImproperEncodingSketcher;
pub use improper_nesting::ImproperNestingSketcher;
pub use incorrect_boolean_literal::IncorrectBooleanLiteralSketcher;
pub use invalid_character::InvalidCharacterSketcher;
pub use invalid_escape_sequence::InvalidEscapeSequenceSketcher;
pub use invalid_number_format::InvalidNumberFormatSketcher;
pub use leading_comma::LeadingCommaSketcher;
pub use missing_comma_between_item::MissingCommaBetweenItemSketcher;
pub use missing_quote::MissingQuoteSketcher;
pub use mixed_type_in_array::MixedTypeInArraySketcher;
pub use null_or_none_literal::NullOrNoneLiteralSketcher;
pub use overly_large_number::OverlyLargeNumberSketcher;
pub use trailing_comma::TrailingCommaSketcher;
pub use unbalanced_bracket::UnbalancedBracketSketcher;
pub use unescaped_quote::UnescapedQuoteSketcher;
pub use unexpected_token::UnexpectedTokenSketcher;

/// `ShapeSketcher` agents that can modify the shape pool (exclusions, shapes).
pub trait ShapeSketcher: UnwindSafe {
    fn name(&self) -> &'static str;
    fn observe(&mut self, c: char, offset: usize);
    fn finalize(&mut self, pool: &mut LocalShapePool) -> Option<SketchNode<'_>>;
    fn box_clone(&self) -> Box<dyn ShapeSketcher>;
}

/// `TokenSketcher` agents that only read the shape pool.
pub trait TokenSketcher: UnwindSafe {
    fn name(&self) -> &'static str;
    fn observe(&mut self, c: char, offset: usize);
    fn finalize<'a>(&'a mut self, pool: &'a LocalShapePool) -> Option<SketchNode<'a>>;
    fn box_clone(&self) -> Box<dyn TokenSketcher>;
}

/// `SketchOrchestrator` provides local-only sketch orchestration.
pub struct SketchOrchestrator<'a> {
    shape_sketchers: Vec<Box<dyn ShapeSketcher + 'a>>,
    token_sketchers: Vec<Box<dyn TokenSketcher + 'a>>,
    pool: Option<LocalShapePool>,
}

#[inline]
fn normalize_node<'a>(mut node: SketchNode<'a>, _input: &str) -> SketchNode<'a> {
    if let SketchPayload::Spans(mut spans) = node.payload {
        dedup_spans(&mut spans);
        node.payload = SketchPayload::Spans(spans);
    }
    node
}

impl Default for SketchOrchestrator<'_> {
    fn default() -> Self {
        SketchOrchestrator {
            // Phase 1 (optional): shape sketchers that can mutate the pool (exclusions, shapes, ancestry hints)
            shape_sketchers: vec![
                // NOTE: add concrete ShapeSketchers here when available, e.g.:
                // Box::new(circular::SomeShapeSketcher::new()),
            ],
            // Phase 2: token sketchers that read the (possibly mutated) pool
            token_sketchers: vec![
                Box::new(CircularReferenceSketcher::new()),
                Box::new(CommentInJsonSketcher::new()),
                Box::new(DoubleCommaSketcher::new()),
                Box::new(DuplicateKeySketcher::new()),
                Box::new(EmptyKeyOrValueSketcher::new()),
                Box::new(ExcessWhitespaceOrNewlineSketcher::new()),
                Box::new(ExtraOrMissingBracketComplexSketcher::new()),
                Box::new(ExtraOrMissingColonSketcher::new()),
                Box::new(ImproperEncodingSketcher::new()),
                Box::new(ImproperNestingSketcher::new()),
                Box::new(IncorrectBooleanLiteralSketcher::new()),
                Box::new(InvalidCharacterSketcher::new()),
                Box::new(InvalidEscapeSequenceSketcher::new()),
                Box::new(InvalidNumberFormatSketcher::new()),
                Box::new(LeadingCommaSketcher::new()),
                Box::new(MissingCommaBetweenItemSketcher::new()),
                Box::new(MissingQuoteSketcher::new()),
                Box::new(MixedTypeInArraySketcher::new()),
                Box::new(NullOrNoneLiteralSketcher::new()),
                Box::new(OverlyLargeNumberSketcher::new()),
                Box::new(TrailingCommaSketcher::new()),
                Box::new(UnbalancedBracketSketcher::new()),
                Box::new(UnescapedQuoteSketcher::new()),
                Box::new(UnexpectedTokenSketcher::new()),
            ],
            pool: None,
        }
    }
}

impl SketchOrchestrator<'_> {
    /// Runs all sketchers and emits basic sketches over the SUPRA pipeline.
    ///
    /// This orchestrates two phases: optional shape-sketchers that may mutate the
    /// local pool, followed by token-sketchers that read from it. Results are
    /// deduplicated and sorted deterministically by kind priority.
    ///
    /// # Panics
    /// Panics if the internal shape pool has not been initialized. This happens
    /// only when accessing the pool via `expect("Shape pool not initialized")` in
    /// either phase orchestration.
    #[instrument(skip_all, fields(stage = "sketch_input"))]
    #[allow(clippy::cognitive_complexity)]
    pub fn run<'b>(&'b mut self, input: &'b str) -> LocalScan<'b> {
        let count = input.len();
        let start = Instant::now();
        let threads = {
            #[cfg(feature = "parallel")]
            {
                rayon::current_num_threads()
            }
            #[cfg(not(feature = "parallel"))]
            {
                1
            }
        };
        let _g = info_span!("sketch_input", items = %count, threads = %threads).entered();

        // Initialize and store pool so we can borrow mutably in phase 1 and immutably in phase 2
        let shape_pool = LocalShapePool::new(input);
        self.pool = Some(shape_pool);

        // ---------------- Phase 1: ShapeSketchers mutate the pool ----------------
        if !self.shape_sketchers.is_empty() {
            let pool_mut = self.pool.as_mut().expect("Shape pool not initialized");
            for sketcher in &mut self.shape_sketchers {
                // Observe over byte offsets for UTF-8 correctness
                for (byte_offset, ch) in input.char_indices() {
                    sketcher.observe(ch, byte_offset);
                }
                // Contain panics from individual sketchers
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    // finalize may mutate the pool; we ignore any return value to avoid borrow escaping the closure
                    let _ = sketcher.finalize(pool_mut);
                }));
            }
        }

        // ---------------- Phase 2: TokenSketchers read the pool ----------------
        let pool_ref = self.pool.as_ref().expect("Shape pool not initialized");
        let mut results: Vec<SketchNode> = Vec::new();
        for sketcher in &mut self.token_sketchers {
            for (byte_offset, ch) in input.char_indices() {
                sketcher.observe(ch, byte_offset);
            }
            // Call finalize directly because it returns a node borrowing `pool_ref`,
            // which cannot safely escape a catch_unwind closure.
            if let Some(sketch) = sketcher.finalize(pool_ref) {
                results.push(normalize_node(sketch, input));
            }
        }

        // Deterministic ordering aligned with shared Kind::priority()
        results.sort_by_key(|n| n.kind().priority());

        let out = LocalScan {
            sketches: results,
            pool: pool_ref,
        };
        info!(elapsed_ms = %start.elapsed().as_millis(), "done");
        out
    }

    /// Returns a reference to the internal shape pool.
    ///
    /// # Panics
    /// Panics if `run` has not been called yet and the pool has not been
    /// initialized, since the pool is created from the provided input during
    /// orchestration.
    #[must_use]
    pub const fn pool(&self) -> &LocalShapePool {
        self.pool.as_ref().expect("Shape pool not initialized")
    }
}

// rectiq-cli/src/types/scan.rs
use crate::types::pool::LocalShapePool;
use rectiq_types::SketchNode;

pub struct LocalScan<'a> {
    pub sketches: Vec<SketchNode<'a>>,
    pub pool: &'a LocalShapePool,
}

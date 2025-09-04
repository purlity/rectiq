// rectiq-cli/src/types/payload.rs
use rectiq_types::Sketch;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FixPayload<'a> {
    pub sketches: &'a [Sketch<'a>],
}

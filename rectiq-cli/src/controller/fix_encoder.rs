// rectiq-cli/src/controller/fix_encoder.rs
use crate::symbolset::model::SymbolSet;
use rectiq_types::{SketchNode, SketchPayload};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum FixEncodingError {
    SerdeError(String),
    MaskingError(String),
}

impl fmt::Display for FixEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerdeError(e) => write!(f, "Serde error: {e}"),
            Self::MaskingError(e) => write!(f, "Masking error: {e}"),
        }
    }
}

impl std::error::Error for FixEncodingError {}

impl From<FixEncodingError> for String {
    fn from(err: FixEncodingError) -> Self {
        err.to_string()
    }
}

pub struct FixEncoder;

impl FixEncoder {
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn encode_all(
        symbolset: &SymbolSet,
        envelopes: &[SketchNode],
    ) -> Result<String, FixEncodingError> {
        println!("📦 FixEncoder CLI - Raw sketch Envelopes: {envelopes:?}");
        let sketches_json: Result<Vec<_>, _> = envelopes
            .iter()
            .map(|env| {
                let format = env.format();
                println!("🧪 Envelope format: {format:?}");
                println!("🧪 Envelope kind: {:?}", env.kind());
                let val = match &env.payload {
                    SketchPayload::Spans(spans) => {
                        json!({ "kind": env.kind(), "payload": { "Spans": spans } })
                    }
                    SketchPayload::Pairs(pairs) => {
                        json!({ "kind": env.kind(), "payload": { "Pairs": pairs } })
                    }
                    SketchPayload::Edges(edges) => {
                        json!({ "kind": env.kind(), "payload": { "Edges": edges } })
                    }
                };
                println!("🧾 Encoded FLAT sketch JSON value: {val}");
                Ok(val)
            })
            .collect();

        let sketches_json = sketches_json?;

        let wrapper = json!({ "sketches": sketches_json });
        let raw = serde_json::to_string(&wrapper)
            .map_err(|e| FixEncodingError::SerdeError(e.to_string()))?;

        symbolset
            .encode_strict(&raw)
            .map_err(FixEncodingError::MaskingError)
    }
}

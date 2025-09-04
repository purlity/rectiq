// rectiq-cli/src/divine/manifest.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct StepManifest {
    pub nonce_hints: Vec<String>,
    pub execution_order: Option<Vec<usize>>,
}

impl StepManifest {
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn from_json(blob: &str) -> Result<Self, String> {
        serde_json::from_str(blob).map_err(|e| format!("Manifest parse error: {e}"))
    }

    #[must_use]
    pub fn execution_order_ref(&self) -> &[usize] {
        self.execution_order.as_ref().map_or(&[], |order| order)
    }

    /// Returns the manifest index sequence in correct order (handles optional override)
    #[must_use]
    pub fn materialize_execution_order(&self) -> Vec<usize> {
        self.execution_order.as_ref().map_or_else(
            || (0..self.nonce_hints.len()).collect(),
            std::clone::Clone::clone,
        )
    }
}

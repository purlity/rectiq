// rectiq-cli/src/symbolset/manager.rs
use crate::types::symbolset::MaskingSymbolSet;
use std::{fs, path::PathBuf};

/// Manages loading of local `SymbolSet` configurations.
pub struct SymbolSetManager;

const DEFAULT_REDACTED_CHAR: char = '•';
const DEFAULT_REPLACEMENTS: &[&str] = &["X", "Y"];

impl SymbolSetManager {
    /// Loads a `MaskingSymbolSet` from the default file location.
    ///
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn load_from_default() -> Result<MaskingSymbolSet, String> {
        let path = Self::default_path();
        let data = fs::read(&path)
            .map_err(|e| format!("Failed to read symbolset from {}: {e}", path.display()))?;

        // Try structured version first
        if let Ok(structured) = serde_json::from_slice::<MaskingSymbolSet>(&data) {
            return Ok(structured);
        }

        // Placeholder for future schema version support

        // Fallback: old base64-only format
        if let Ok(s) = serde_json::from_slice::<String>(&data) {
            return Ok(MaskingSymbolSet {
                base64_key: s,
                redacted_char: DEFAULT_REDACTED_CHAR,
                replacement_patterns: DEFAULT_REPLACEMENTS
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            });
        }

        Err("❌ Failed to parse symbolset file in any known format.".to_string())
    }

    /// Returns the default path to the symbolset configuration file.
    ///
    /// # Panics
    /// This function will panic if the home directory cannot be determined.
    #[must_use]
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".rectiq")
            .join("symbolset.json")
    }
}

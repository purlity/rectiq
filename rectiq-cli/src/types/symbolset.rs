// rectiq-cli/src/types/symbolset.rs
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingSymbolSet {
    pub base64_key: String,
    pub redacted_char: char,
    pub replacement_patterns: Vec<String>,
}

impl MaskingSymbolSet {
    /// # Panics
    /// Panics if the home directory cannot be determined.
    #[must_use]
    pub fn default_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not determine home directory");
        home.join(".rectiq").join("symbolset.json")
    }

    #[must_use]
    pub fn load_from_disk() -> Option<Self> {
        let path = Self::default_path();
        if !path.exists() {
            return None;
        }

        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn save_to_disk(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

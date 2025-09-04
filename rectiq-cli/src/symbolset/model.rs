// rectiq-cli/src/symbolset/model.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolSet {
    pub map: HashMap<char, String>,
}

impl SymbolSet {
    #[must_use]
    /// Encodes input using the symbol set.
    ///
    /// # Panics
    /// Panics if any character in the input is unmapped, due to the `.expect()` call in `encode_internal`.
    pub fn encode(&self, input: &str) -> String {
        self.encode_internal(input, true)
            .expect("Encode failed due to missing mapping")
    }

    /// Encodes input and returns error if any symbol is unmapped (strict mode).
    ///
    /// # Errors
    /// Returns an error when the input payload fails validation.
    pub fn encode_strict(&self, input: &str) -> Result<String, String> {
        self.encode_internal(input, true)
    }

    /// Internal encode logic for both encode (strict) and `encode_full` (non-strict)
    fn encode_internal(&self, input: &str, strict: bool) -> Result<String, String> {
        if self.map.is_empty() {
            // Unified warning
            eprintln!("⚠️  SymbolSet.encode(): map is empty — encoding will be a no-op");
            debug_assert!(false, "SymbolSet.encode(): map is empty, encoding bypassed");
        }
        let mut out = String::new();
        for c in input.chars() {
            if let Some(s) = self.map.get(&c) {
                out.push_str(s);
            } else {
                if strict {
                    return Err(format!("❌ SymbolSet missing mapping for char: '{c}'"));
                }
                out.push(c);
            }
        }
        Ok(out)
    }

    #[must_use]
    /// Constructs a `SymbolSet` from a forward map.
    pub const fn from_map(map: HashMap<char, String>) -> Self {
        Self { map }
    }
}

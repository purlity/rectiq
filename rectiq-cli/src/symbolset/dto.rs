// rectiq-cli/src/symbolset/dto.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SymbolSetMetadata {
    pub user_id: String,
    pub required: Vec<char>,
    pub masked: Vec<char>,
    pub hash: String,
}

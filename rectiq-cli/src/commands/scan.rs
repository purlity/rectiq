// rectiq-cli/src/commands/scan.rs
use crate::{discover_files, scan};
use anyhow::Result;
use std::path::Path;

/// # Errors
/// Returns an error when the input payload fails validation.
pub fn scan_mode(dir: &str) -> Result<()> {
    let paths = discover_files(Path::new(dir))?;
    for path in paths {
        let input = std::fs::read_to_string(&path)?;
        let _ = scan(&input);
    }
    Ok(())
}

// rectiq-cli/src/commands/fix.rs
use crate::run;
use std::fs;

/// Run the fixer on the provided file path.
///
/// # Panics
/// Panics if the input file cannot be read.
pub fn fix_mode(file_path: &str, user_id: &str) {
    let input = fs::read_to_string(file_path).expect("Failed to read input file.");
    match run(&input, user_id) {
        Ok(output) => println!("✅ Fixed output:\n{output}"),
        Err(err) => println!("{err}"),
    }
}

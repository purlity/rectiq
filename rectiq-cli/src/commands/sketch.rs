// rectiq-cli/src/commands/sketch.rs
use crate::scan;
use std::fs;

/// Run the sketch analyzer on the provided file path.
///
/// # Panics
/// Panics if the input file cannot be read.
pub fn sketch_mode(file_path: &str) {
    let input = fs::read_to_string(file_path).expect("Failed to read input file.");
    let issues = scan(&input);
    for sketch in issues {
        println!("🧠 Sketch: {sketch:?}");
    }
}

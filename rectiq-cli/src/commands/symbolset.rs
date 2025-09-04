// rectiq-cli/src/commands/symbolset.rs
use crate::{symbolset::init::generate_random_symbolset, types::symbolset::MaskingSymbolSet};

/// Initialize a new `SymbolSet` and save it to disk.
///
/// # Panics
/// Panics if the `SymbolSet` fails to save after initialization.
pub fn init_symbolset() {
    let set: MaskingSymbolSet = generate_random_symbolset();
    let path = MaskingSymbolSet::default_path();
    match set.save_to_disk() {
        Ok(()) => {
            if path.exists() {
                println!("✅ SymbolSet initialized and saved to ~/.rectiq/symbolset.json");
            } else {
                panic!("❌ SymbolSet was not saved as expected.");
            }
        }
        Err(e) => eprintln!("❌ Failed to save SymbolSet: {e}"),
    }
}

/// View the current `SymbolSet` on disk, if any.
pub fn view_symbolset() {
    match MaskingSymbolSet::load_from_disk() {
        Some(set) => {
            println!("🔐 Current SymbolSet:");
            println!("• Redacted char: '{}'", set.redacted_char);
            println!("• Replacement patterns:");
        }
        None => {
            println!("⚠️  No SymbolSet found. You can initialize one with: rectiq symbolset-init");
        }
    }
}

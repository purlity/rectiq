// rectiq-cli/src/symbolset/init.rs
use crate::{types::symbolset::MaskingSymbolSet};
use base64::Engine as _;
use base64::engine::general_purpose;
use getrandom::u64;
use rectiq_crypto::gen_bytes;

pub const CLI_CHARSET: &[char] = &[
    '𐑧', '𐑨', '𐑩', '𐑪', '𐑫', '𐑬', '𐑭', '𐑮', '𐑯', '𐑰', '𐑱', '𐑲', '𐑳', '𐑴', '𐑵', '𐑶', '𐑷', '𐑸', '𐑹',
    '𐑺',
];

/// Generate a random `MaskingSymbolSet` using OS CSPRNG.
///
/// # Panics
/// Panics if the operating system random number generator fails
/// or if the character set length cannot be represented in `u64`/`usize`.
#[must_use]
pub fn generate_random_symbolset() -> MaskingSymbolSet {
    // Generate base64 key
    let key_bytes = gen_bytes::<32>();
    let base64_key = general_purpose::STANDARD.encode(key_bytes);

    // Select redacted_char using getrandom::u64 and index calculation
    let len_u64 = u64::try_from(CLI_CHARSET.len()).expect("charset len fits in u64");
    let idx =
        usize::try_from(u64().expect("OS RNG failure") % len_u64).expect("index fits in usize");
    let redacted_char = CLI_CHARSET[idx];

    let mut patterns = Vec::new();
    for _ in 0..5 {
        let pattern: String = (0..3)
            .map(|_| {
                let len_u64 = u64::try_from(CLI_CHARSET.len()).expect("charset len fits in u64");
                let idx = usize::try_from(u64().expect("OS RNG failure") % len_u64)
                    .expect("index fits in usize");
                CLI_CHARSET[idx]
            })
            .collect();
        patterns.push(pattern);
    }

    MaskingSymbolSet {
        base64_key,
        redacted_char,
        replacement_patterns: patterns,
    }
}

// rectiq-cli/src/pipeline/lexer.rs

// Minimal JSON-like lexer for the Rectiq sketcher pipeline.

// This module provides a simple lexer that tokenizes JSON-like input into a flat stream of tokens.
// It is designed for use in the Rectiq sketcher pipeline, which requires fast, lossless tokenization
// of the entire input, including whitespace and comments. The lexer does not attempt to validate
// strict JSON syntax (e.g., numbers may be malformed, unescaped characters in strings are not checked,
// and unknown tokens are possible for any unrecognized byte or malformed sequence).

// ## Role and Invariants
// - The lexer covers the entire input byte-for-byte, producing a series of `Token`s that together
//   span the input with no gaps or overlaps.
// - No AST or higher-level structure is built; this is a pure lexer.
// - All whitespace and comments are tokenized and preserved.
// - Any unknown or invalid bytes/sequences are emitted as `TokKind::Unknown` tokens.
// - The final token is always `TokKind::Eof` at the end of the input.

// ## Tokenization Notes
// - Recognizes JSON punctuation, literals, strings (with basic escaping), numbers (loosely),
//   whitespace, and both `//` and `/* ... */` comments.
// - Unknown or invalid sequences are not fatal; they're simply emitted as `Unknown`.
// - The lexer is tolerant, but not validating: e.g., malformed numbers or unterminated strings
//   will be `Unknown`.
// - All offsets are byte indices into the original input.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The different kinds of tokens recognized by the lexer for JSON-like syntax.
pub enum TokKind {
    /// `{` Left curly brace.
    LBrace,
    /// `}` Right curly brace.
    RBrace,
    /// `[` Left square bracket.
    LBracket,
    /// `]` Right square bracket.
    RBracket,
    /// `:` Colon separator.
    Colon,
    /// `,` Comma separator.
    Comma,
    /// `String` literal, e.g., `"foo"`. May be malformed (unterminated = `Unknown`).
    StringLit,
    /// Number literal, e.g., `42`, `-1.23e+4`. May be loosely validated.
    NumberLit,
    /// The literal `true`.
    True,
    /// The literal `false`.
    False,
    /// The literal `null`.
    Null,
    /// Whitespace (spaces, tabs, newlines, carriage returns).
    Whitespace,
    /// Comment, either `// ...` or `/* ... */`.
    Comment,
    /// Unknown or unrecognized token (invalid byte or malformed construct).
    Unknown,
    /// End of file/input marker.
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A token as produced by the lexer, with its kind and byte offsets.
///
/// - `kind`: The type of token (see [`TokKind`]).
/// - `start`: The starting byte offset of the token in the input.
/// - `end`: The exclusive end byte offset (i.e., the token covers `input[start..end]`).
pub struct Token {
    pub kind: TokKind,
    pub start: usize,
    pub end: usize, // exclusive
}

/// Scans the input and produces a vector of tokens covering the entire input byte range.
///
/// This lexer uses a single-pass, byte-oriented strategy to recognize JSON-like tokens, loosely
/// validating numbers and strings, and accepting both C-style (`//`, `/* ... */`) comments.
/// It does not enforce strict JSON validation: malformed numbers, unterminated strings/comments,
/// or any unknown bytes are tokenized as `TokKind::Unknown`.
///
/// All whitespace and comments are preserved as tokens.
/// The returned tokens together span the entire input with no gaps or overlaps.
/// The final token is always an `Eof` marker at the end of the input.
///
/// # Limitations
/// - Not a validating JSON lexer; unknown tokens are possible.
/// - `String`s are only checked for closing quotes, not for full JSON escape validity.
/// - Numbers are parsed loosely (e.g., may accept invalid forms).
///
/// # Arguments
/// * `input` - The input string to tokenize.
///
/// # Returns
/// A vector of `Token`s covering the entire input.
#[must_use]
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    let len = bytes.len();
    while i < len {
        let start = i;
        let b = bytes[i];
        let token = match b {
            b'{' => Token {
                kind: TokKind::LBrace,
                start,
                end: start + 1,
            },
            b'}' => Token {
                kind: TokKind::RBrace,
                start,
                end: start + 1,
            },
            b'[' => Token {
                kind: TokKind::LBracket,
                start,
                end: start + 1,
            },
            b']' => Token {
                kind: TokKind::RBracket,
                start,
                end: start + 1,
            },
            b':' => Token {
                kind: TokKind::Colon,
                start,
                end: start + 1,
            },
            b',' => Token {
                kind: TokKind::Comma,
                start,
                end: start + 1,
            },
            b'\"' => {
                let mut idx = start + 1;
                let mut escaped = false;
                while idx < len {
                    let c = bytes[idx];
                    if escaped {
                        escaped = false;
                        idx += 1;
                        continue;
                    }
                    match c {
                        b'\\' => {
                            escaped = true;
                            idx += 1;
                        }
                        b'\"' => {
                            idx += 1;
                            break;
                        }
                        _ => idx += 1,
                    }
                }
                if idx <= len && bytes.get(idx - 1) == Some(&b'\"') {
                    Token {
                        kind: TokKind::StringLit,
                        start,
                        end: idx,
                    }
                } else {
                    Token {
                        kind: TokKind::Unknown,
                        start,
                        end: start + 1,
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                let mut idx = start;
                if bytes[idx] == b'-' {
                    idx += 1;
                    if idx >= len || !bytes[idx].is_ascii_digit() {
                        Token {
                            kind: TokKind::Unknown,
                            start,
                            end: start + 1,
                        }
                    } else {
                        idx += 0; // fallthrough to digits below
                        while idx < len && bytes[idx].is_ascii_digit() {
                            idx += 1;
                        }
                        if idx < len && bytes[idx] == b'.' {
                            idx += 1;
                            while idx < len && bytes[idx].is_ascii_digit() {
                                idx += 1;
                            }
                        }
                        if idx < len && (bytes[idx] == b'e' || bytes[idx] == b'E') {
                            idx += 1;
                            if idx < len && (bytes[idx] == b'+' || bytes[idx] == b'-') {
                                idx += 1;
                            }
                            while idx < len && bytes[idx].is_ascii_digit() {
                                idx += 1;
                            }
                        }
                        Token {
                            kind: TokKind::NumberLit,
                            start,
                            end: idx,
                        }
                    }
                } else {
                    // starts with digit
                    if bytes[idx] == b'0' {
                        idx += 1;
                    } else {
                        while idx < len && bytes[idx].is_ascii_digit() {
                            idx += 1;
                        }
                    }
                    if idx < len && bytes[idx] == b'.' {
                        idx += 1;
                        while idx < len && bytes[idx].is_ascii_digit() {
                            idx += 1;
                        }
                    }
                    if idx < len && (bytes[idx] == b'e' || bytes[idx] == b'E') {
                        idx += 1;
                        if idx < len && (bytes[idx] == b'+' || bytes[idx] == b'-') {
                            idx += 1;
                        }
                        while idx < len && bytes[idx].is_ascii_digit() {
                            idx += 1;
                        }
                    }
                    Token {
                        kind: TokKind::NumberLit,
                        start,
                        end: idx,
                    }
                }
            }
            b't' if input[start..].starts_with("true") => Token {
                kind: TokKind::True,
                start,
                end: start + 4,
            },
            b'f' if input[start..].starts_with("false") => Token {
                kind: TokKind::False,
                start,
                end: start + 5,
            },
            b'n' if input[start..].starts_with("null") => Token {
                kind: TokKind::Null,
                start,
                end: start + 4,
            },
            b'/' => {
                if input[start..].starts_with("//") {
                    let mut idx = start + 2;
                    while idx < len && bytes[idx] != b'\n' {
                        idx += 1;
                    }
                    Token {
                        kind: TokKind::Comment,
                        start,
                        end: idx,
                    }
                } else if input[start..].starts_with("/*") {
                    let mut idx = start + 2;
                    while idx + 1 < len {
                        if bytes[idx] == b'*' && bytes[idx + 1] == b'/' {
                            idx += 2;
                            break;
                        }
                        idx += 1;
                    }
                    Token {
                        kind: TokKind::Comment,
                        start,
                        end: idx.min(len),
                    }
                } else {
                    Token {
                        kind: TokKind::Unknown,
                        start,
                        end: start + 1,
                    }
                }
            }
            b' ' | b'\n' | b'\t' | b'\r' => {
                let mut idx = start + 1;
                while idx < len && matches!(bytes[idx], b' ' | b'\n' | b'\t' | b'\r') {
                    idx += 1;
                }
                Token {
                    kind: TokKind::Whitespace,
                    start,
                    end: idx,
                }
            }
            _ => Token {
                kind: TokKind::Unknown,
                start,
                end: start + 1,
            },
        };
        i = token.end;
        tokens.push(token);
    }
    tokens.push(Token {
        kind: TokKind::Eof,
        start: len,
        end: len,
    });
    tokens
}

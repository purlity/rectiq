// rectiq-cli/src/pipeline/skeleton.rs

// Skeleton builder for the SUPRA pipeline (tokens → minimal structure).

// Role:
// - Consume the flat token stream and reconstruct a *minimal* structural skeleton:
//   bracket stack, object key/value pairs, array elements, and culprit indices for
//   bracket mismatches. This is **not** a full AST; it is a context substrate for
//   detectors and lattice classification.

// Invariants:
// - Bracket entry/exit events are tracked deterministically in `frames`.
// - `obj_pairs` holds coarse key/value token index spans; keys are string tokens when present.
// - `arr_elems` holds coarse element token index spans inside arrays.
// - `bracket_mismatches` records the byte index of unmatched closers or dangling openers.

// Privacy/Truth:
// - The skeleton never mutates input, and only slices for key snippets on demand in `path_at`.
//   This remains local (CLI-side) and complies with our Zero‑Knowledge boundary.
use std::borrow::Cow;

use super::lexer::{TokKind, Token};

#[inline]
const fn is_insignificant(kind: TokKind) -> bool {
    matches!(kind, TokKind::Whitespace | TokKind::Comment)
}

#[inline]
fn prev_significant_from(tokens: &[Token], mut i: usize) -> Option<usize> {
    loop {
        let k = tokens[i].kind;
        if !is_insignificant(k) {
            return Some(i);
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }
    None
}

/// Structural frame kinds for the context stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Obj,
    Arr,
}

/// A frame entry event with optional key snippet (when known in objects).
#[derive(Debug, Clone)]
pub struct ContextFrame<'a> {
    pub kind: FrameKind,
    pub start: usize,
    pub key_snippet: Option<Cow<'a, str>>, // for obj keys when known
}

/// A coarse object key/value pair expressed as token index spans.
#[derive(Debug, Clone)]
pub struct ObjPair {
    pub key_span: (usize, usize),   // token range
    pub value_span: (usize, usize), // token range
}

/// A coarse array element expressed as a token index span.
#[derive(Debug, Clone)]
pub struct ArrayElem {
    pub span: (usize, usize),
}

/// Minimal structural substrate derived from tokens.
#[derive(Debug, Default, Clone)]
pub struct Skeleton<'a> {
    pub frames: Vec<ContextFrame<'a>>,  // chronological entry events
    pub obj_pairs: Vec<ObjPair>,        // discovered key/value pairs
    pub arr_elems: Vec<ArrayElem>,      // array elements
    pub bracket_mismatches: Vec<usize>, // culprit byte indices
}

/// Build skeleton from tokens and input
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
#[must_use]
pub fn build_skeleton(input: &str, tokens: &[Token]) -> Skeleton<'static> {
    #[derive(Debug)]
    struct ObjState {
        key_tok: Option<usize>,
        value_start: Option<usize>,
        value_depth: usize,
    }
    #[derive(Debug)]
    struct ArrState {
        elem_start: Option<usize>,
        elem_depth: usize,
    }
    #[derive(Debug)]
    enum State {
        Obj(ObjState),
        Arr(ArrState),
    }

    let mut skel = Skeleton::default();
    let mut stack: Vec<State> = Vec::new();

    for (i, tok) in tokens.iter().enumerate() {
        match tok.kind {
            TokKind::LBrace => {
                // If we're inside an object and expecting a value, mark value start at this token
                let cur_depth = stack.len();
                if let Some(State::Obj(obj)) = stack.last_mut()
                    && obj.key_tok.is_some()
                    && obj.value_start.is_none()
                {
                    obj.value_start = Some(i);
                    obj.value_depth = cur_depth;
                }

                // Push new object frame
                stack.push(State::Obj(ObjState {
                    key_tok: None,
                    value_start: None,
                    value_depth: 0,
                }));
                skel.frames.push(ContextFrame {
                    kind: FrameKind::Obj,
                    start: tok.start,
                    key_snippet: None,
                });

                // If we're inside an array, starting a new element here
                if let Some(State::Arr(arr)) = stack.iter_mut().rev().nth(1)
                    && arr.elem_start.is_none()
                {
                    arr.elem_start = Some(i);
                }
            }
            TokKind::LBracket => {
                // If we're inside an object and expecting a value, mark value start at this token
                let cur_depth = stack.len();
                if let Some(State::Obj(obj)) = stack.last_mut()
                    && obj.key_tok.is_some()
                    && obj.value_start.is_none()
                {
                    obj.value_start = Some(i);
                    obj.value_depth = cur_depth;
                }

                // Push new array frame
                stack.push(State::Arr(ArrState {
                    elem_start: None,
                    elem_depth: stack.len(),
                }));
                skel.frames.push(ContextFrame {
                    kind: FrameKind::Arr,
                    start: tok.start,
                    key_snippet: None,
                });

                // If we're inside a parent array, starting a new element here
                if let Some(State::Arr(arr)) = stack.iter_mut().rev().nth(1)
                    && arr.elem_start.is_none()
                {
                    arr.elem_start = Some(i);
                }
            }
            TokKind::RBrace => {
                if let Some(State::Obj(obj)) = stack.pop() {
                    if let (Some(k), Some(vs)) = (obj.key_tok, obj.value_start) {
                        let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                            .unwrap_or_else(|| i.saturating_sub(1));
                        skel.obj_pairs.push(ObjPair {
                            key_span: (k, k + 1),
                            value_span: (vs, end_idx + 1),
                        });
                    }
                } else {
                    skel.bracket_mismatches.push(tok.start);
                }
                if let Some(State::Arr(arr)) = stack.last_mut()
                    && let Some(vs) = arr.elem_start
                {
                    let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                        .unwrap_or_else(|| i.saturating_sub(1));
                    skel.arr_elems.push(ArrayElem {
                        span: (vs, end_idx + 1),
                    });
                    arr.elem_start = None;
                }
            }
            TokKind::RBracket => {
                if let Some(State::Arr(arr)) = stack.pop() {
                    if let Some(vs) = arr.elem_start {
                        let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                            .unwrap_or_else(|| i.saturating_sub(1));
                        skel.arr_elems.push(ArrayElem {
                            span: (vs, end_idx + 1),
                        });
                    }
                } else {
                    skel.bracket_mismatches.push(tok.start);
                }
                if let Some(State::Arr(arr)) = stack.last_mut()
                    && let Some(vs) = arr.elem_start
                {
                    let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                        .unwrap_or_else(|| i.saturating_sub(1));
                    skel.arr_elems.push(ArrayElem {
                        span: (vs, end_idx + 1),
                    });
                    arr.elem_start = None;
                }
            }
            TokKind::StringLit => {
                let cur_depth = stack.len();
                if let Some(State::Obj(obj)) = stack.last_mut() {
                    if obj.key_tok.is_none() {
                        obj.key_tok = Some(i);
                    } else if let Some(State::Arr(arr)) = stack.last_mut()
                        && arr.elem_start.is_none()
                    {
                        arr.elem_start = Some(i);
                        arr.elem_depth = cur_depth;
                    }
                } else if let Some(State::Arr(arr)) = stack.last_mut()
                    && arr.elem_start.is_none()
                {
                    arr.elem_start = Some(i);
                    arr.elem_depth = cur_depth;
                }
            }
            TokKind::Colon => {
                if let Some(State::Obj(obj)) = stack.last_mut()
                    && obj.key_tok.is_some()
                {
                    obj.value_start = None; // wait for value token
                }
            }
            TokKind::Comma => {
                let cur_depth = stack.len();
                match stack.last_mut() {
                    Some(State::Obj(obj)) => {
                        if let (Some(k), Some(vs)) = (obj.key_tok, obj.value_start)
                            && obj.value_depth == cur_depth
                        {
                            let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                                .unwrap_or_else(|| i.saturating_sub(1));
                            skel.obj_pairs.push(ObjPair {
                                key_span: (k, k + 1),
                                value_span: (vs, end_idx + 1),
                            });
                            obj.key_tok = None;
                            obj.value_start = None;
                        }
                    }
                    Some(State::Arr(arr)) => {
                        if let Some(vs) = arr.elem_start
                            && arr.elem_depth == cur_depth
                        {
                            let end_idx = prev_significant_from(tokens, i.saturating_sub(1))
                                .unwrap_or_else(|| i.saturating_sub(1));
                            skel.arr_elems.push(ArrayElem {
                                span: (vs, end_idx + 1),
                            });
                            arr.elem_start = None;
                        }
                    }
                    _ => {}
                }
            }
            TokKind::Whitespace | TokKind::Comment => {}
            _ => {
                let cur_depth = stack.len();
                match stack.last_mut() {
                    Some(State::Obj(obj)) => {
                        if obj.key_tok.is_some() && obj.value_start.is_none() {
                            obj.value_start = Some(i);
                            obj.value_depth = cur_depth;
                        }
                    }
                    Some(State::Arr(arr)) => {
                        if arr.elem_start.is_none() {
                            arr.elem_start = Some(i);
                            arr.elem_depth = cur_depth;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // finalize dangling at EOF
    if let Some(state) = stack.last() {
        match state {
            State::Obj(obj) => {
                if let (Some(k), Some(vs)) = (obj.key_tok, obj.value_start) {
                    let end_idx = prev_significant_from(tokens, tokens.len().saturating_sub(1))
                        .unwrap_or_else(|| tokens.len().saturating_sub(1));
                    skel.obj_pairs.push(ObjPair {
                        key_span: (k, k + 1),
                        value_span: (vs, end_idx + 1),
                    });
                }
            }
            State::Arr(arr) => {
                if let Some(vs) = arr.elem_start {
                    let end_idx = prev_significant_from(tokens, tokens.len().saturating_sub(1))
                        .unwrap_or_else(|| tokens.len().saturating_sub(1));
                    skel.arr_elems.push(ArrayElem {
                        span: (vs, end_idx + 1),
                    });
                }
            }
        }
    }

    // fill key snippets for frames -- naive: none for now
    let _ = input; // avoid unused
    skel
}

impl Skeleton<'static> {
    /// Returns the parent key path (outermost→innermost) and depth for a given token index.
    /// Uses `input` only to slice the string contents of key tokens for human‑readable context.
    #[must_use]
    #[inline]
    pub fn path_at(
        &self,
        tokens: &[Token],
        input: &str,
        tok_idx: usize,
    ) -> (Vec<Cow<'static, str>>, u8) {
        let mut pairs: Vec<(usize, &ObjPair)> = Vec::new();
        for pair in &self.obj_pairs {
            if pair.value_span.0 <= tok_idx && tok_idx < pair.value_span.1 {
                pairs.push((pair.value_span.0, pair));
            }
        }
        pairs.sort_by_key(|p| p.0);
        let mut keys = Vec::new();
        for (_, pair) in pairs {
            let s = tokens[pair.key_span.0].start + 1;
            let e = tokens[pair.key_span.1 - 1].end - 1;
            if e >= s && e <= input.len() {
                let snippet = &input[s..e];
                keys.push(Cow::Owned(snippet.to_string()));
            }
        }
        #[allow(clippy::cast_possible_truncation)]
        let depth = keys.len() as u8;
        (keys, depth)
    }
}

//! `:wat::core::string::*` + `:wat::core::regex::*` — string basics.
//!
//! Follows the `:wat::core::i64::*` precedent: per-type operations live
//! in their own sub-namespace under `:wat::core::`. Keeps the top-level
//! `:wat::core::*` reserved for polymorphic forms (`=`, `first`, `map`,
//! `length` on Vec, etc.).
//!
//! Char-oriented, not byte-oriented. `length` returns `chars().count()`;
//! `split` uses `&str::split` which is UTF-8 safe; substring primitives
//! would be added as `char_index`-based when a caller needs them.
//!
//! Regex lives next door at `:wat::core::regex::*` because the `regex`
//! crate is its own concern — a wat-rs deployment that didn't want the
//! regex dep could feature-gate this module separately in a future
//! refactor.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, SymbolTable, Value};
use std::sync::Arc;

/// `(:wat::core::string::contains? haystack needle)` → `:bool`.
pub fn eval_string_contains(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, needle) = two_strings(":wat::core::string::contains?", args, env, sym)?;
    Ok(Value::bool(hay.contains(needle.as_str())))
}

/// `(:wat::core::string::starts-with? haystack prefix)` → `:bool`.
pub fn eval_string_starts_with(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, prefix) = two_strings(":wat::core::string::starts-with?", args, env, sym)?;
    Ok(Value::bool(hay.starts_with(prefix.as_str())))
}

/// `(:wat::core::string::ends-with? haystack suffix)` → `:bool`.
pub fn eval_string_ends_with(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, suffix) = two_strings(":wat::core::string::ends-with?", args, env, sym)?;
    Ok(Value::bool(hay.ends_with(suffix.as_str())))
}

/// `(:wat::core::string::length s)` → `:i64`.
///
/// Unicode scalar count — matches the user's mental model of "string
/// length" for scripts that use grapheme-sized characters. For byte
/// length, encode through `:Vec<u8>` and use that vec's `length`.
pub fn eval_string_length(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::length", args, env, sym)?;
    Ok(Value::i64(s.chars().count() as i64))
}

/// `(:wat::core::string::trim s)` → `:String`.
pub fn eval_string_trim(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::trim", args, env, sym)?;
    Ok(Value::String(Arc::new(s.trim().to_string())))
}

/// `(:wat::core::string::split haystack sep)` → `:Vec<String>`.
///
/// Splits every occurrence of `sep`. An empty `sep` — the edge case
/// `str::split("")` would degenerate to per-char — is refused as a
/// MalformedForm: almost always a bug, never obvious what the caller
/// wanted. Callers who genuinely want per-char iteration can encode
/// through `Vec<u8>` via the IO layer.
pub fn eval_string_split(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::split";
    let (hay, sep) = two_strings(OP, args, env, sym)?;
    if sep.is_empty() {
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: "separator must not be empty".into(),
        });
    }
    let pieces: Vec<Value> = hay
        .split(sep.as_str())
        .map(|s| Value::String(Arc::new(s.to_string())))
        .collect();
    Ok(Value::Vec(Arc::new(pieces)))
}

/// `(:wat::core::string::join sep pieces)` → `:String`.
///
/// Signature order matches Rust's `Vec::<String>::join(&sep)`: separator
/// first (the uniform thing), pieces second (the per-call thing).
pub fn eval_string_join(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::join";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let sep = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };
    let pieces = match eval(&args[1], env, sym)? {
        Value::Vec(items) => items,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "Vec<String>",
                got: other.type_name(),
            });
        }
    };
    let mut pieces_owned: Vec<String> = Vec::with_capacity(pieces.len());
    for item in pieces.iter() {
        match item {
            Value::String(s) => pieces_owned.push((**s).clone()),
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: other.type_name(),
                });
            }
        }
    }
    Ok(Value::String(Arc::new(pieces_owned.join(&sep))))
}

// ─── regex ───────────────────────────────────────────────────────────────

/// `(:wat::core::regex::matches? pattern haystack)` → `:bool`.
///
/// True iff `pattern` matches anywhere in `haystack`. Not anchored — use
/// `^...$` inside the pattern for full-string match. Pattern compile
/// failure surfaces as MalformedForm; typical user errors (unbalanced
/// bracket, invalid escape) get the regex crate's own diagnostic.
pub fn eval_regex_matches(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::regex::matches?";
    let (pattern, haystack) = two_strings(OP, args, env, sym)?;
    let re = regex::Regex::new(pattern.as_str()).map_err(|e| RuntimeError::MalformedForm {
        head: OP.into(),
        reason: format!("invalid regex: {}", e),
    })?;
    Ok(Value::bool(re.is_match(haystack.as_str())))
}

// ─── helpers ─────────────────────────────────────────────────────────────

fn one_string(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    match eval(&args[0], env, sym)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: other.type_name(),
        }),
    }
}

fn two_strings(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<(String, String), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };
    let b = match eval(&args[1], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: other.type_name(),
            });
        }
    };
    Ok((a, b))
}

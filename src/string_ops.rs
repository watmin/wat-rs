//! `:wat::core::string::*` + `:wat::core::regex::*` + `:wat::core::uuid::*`
//! — string basics and substrate-level UUID minting.
//!
//! Follows the `:wat::core::i64` precedent: per-type operations live
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
//!
//! UUID — `:wat::core::uuid::v4` (arc 206 slice 1) lives here because it
//! returns `:wat::core::String` and belongs to the same "String-returning
//! substrate utility" category as the string ops. No opaque type; no dep
//! beyond `wat_edn`'s `mint` feature (already enabled by the `wat` crate).

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
/// length, encode through `:wat::core::Vector<u8>` and use that vec's `length`.
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

/// `(:wat::core::string::split haystack sep)` → `:wat::core::Vector<String>`.
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
            span: args[1].span().clone(),
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
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(crate::span::Span::unknown);
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
            span,
        });
    }
    let sep = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: other.type_name(),
                span: args[0].span().clone(),
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
                span: args[1].span().clone(),
            });
        }
    };
    let mut pieces_owned: Vec<String> = Vec::with_capacity(pieces.len());
    for item in pieces.iter() {
        match item {
            Value::String(s) => pieces_owned.push((**s).clone()),
            other => {
                // arc 138: no span — Vec element iteration over Values; per-element WatAST span unavailable
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: other.type_name(),
                    span: crate::span::Span::unknown(),
                });
            }
        }
    }
    Ok(Value::String(Arc::new(pieces_owned.join(&sep))))
}

/// `(:wat::core::string::concat s1 s2 ... sn)` → `:String`.
///
/// Variadic concatenation. Differs from `join` in that there's no
/// separator and the args are passed positionally rather than packed
/// into a `Vec<String>` — the natural form for "stitch a few strings
/// together at the call site." Equivalent to
/// `(:wat::core::string::join "" (:wat::core::Vector :String s1 s2 ...))`
/// but spares the caller the Vec ceremony when concatenation is the
/// goal and the arity is fixed at the call site.
///
/// Arity: 1+. Empty arg list errors (the empty string has no useful
/// concat semantics worth special-casing).
pub fn eval_string_concat(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::concat";
    if args.is_empty() {
        // arc 138: no span — args is empty, no WatAST span available; cross-file broadening out of scope
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: 0,
            span: crate::span::Span::unknown(),
        });
    }
    let mut total = 0usize;
    let mut pieces: Vec<Arc<String>> = Vec::with_capacity(args.len());
    for arg in args {
        match eval(arg, env, sym)? {
            Value::String(s) => {
                total += s.len();
                pieces.push(s);
            }
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: other.type_name(),
                    span: arg.span().clone(),
                });
            }
        }
    }
    let mut out = String::with_capacity(total);
    for p in &pieces {
        out.push_str(p);
    }
    Ok(Value::String(Arc::new(out)))
}

// ─── uuid ────────────────────────────────────────────────────────────────

/// `(:wat::core::uuid::v4)` → `:wat::core::String`.
///
/// Mints a fresh v4 UUID on every call via `wat_edn::new_uuid_v4` (arc 092).
/// Returns the canonical 8-4-4-4-12 hyphenated hex representation —
/// always 36 chars, always lowercase, always hyphenated at positions 8, 13,
/// 18, 23.
///
/// Arity-0; any arguments are a runtime ArityMismatch.
///
/// Arc 206 slice 1 — substrate promotion. Available to all wat code without
/// a `:wat::telemetry` dep. `:wat::telemetry::uuid::v4` delegates to the
/// same `wat_edn::new_uuid_v4` source via its own Rust shim; both paths are
/// independently valid.
pub fn eval_uuid_v4(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::uuid::v4";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
            span: args[0].span().clone(),
        });
    }
    let id = wat_edn::new_uuid_v4().to_string();
    Ok(Value::String(Arc::new(id)))
}

/// `(:wat::core::uuid::v5 namespace name)` → `:wat::core::String`.
///
/// Mints a deterministic v5 (SHA-1-based) UUID from a `namespace` UUID string
/// and a `name` string. Returns the canonical 8-4-4-4-12 hyphenated hex
/// representation — always 36 chars, always lowercase.
///
/// `namespace` must be a canonical 36-char UUID string (e.g., the RFC 4122
/// DNS namespace `"6ba7b810-9dad-11d1-80b4-00c04fd430c8"`). An invalid
/// namespace panics with `assertion-failed!` and a clear diagnostic.
///
/// Arity-2; any other argument count is a runtime ArityMismatch.
///
/// Arc 206 slice 1.5 — substrate promotion. Available to all wat code without
/// a `:wat::telemetry` dep. Deterministic: same (namespace, name) always
/// produces the same UUID.
pub fn eval_uuid_v5(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::uuid::v5";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
            span: if args.is_empty() {
                crate::span::Span::unknown()
            } else {
                args[0].span().clone()
            },
        });
    }
    let ns_val = eval(&args[0], env, sym)?;
    let name_val = eval(&args[1], env, sym)?;
    let ns_str = match &ns_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String".into(),
                got: ns_val.type_name().into(),
                span: args[0].span().clone(),
            });
        }
    };
    let name_str = match &name_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String".into(),
                got: name_val.type_name().into(),
                span: args[1].span().clone(),
            });
        }
    };
    let ns_uuid = uuid::Uuid::parse_str(&ns_str).unwrap_or_else(|_| {
        panic!(
            "assertion-failed! uuid::v5: namespace must be a canonical UUID string, got: {:?}",
            ns_str
        )
    });
    let id = wat_edn::new_uuid_v5(ns_uuid, &name_str).to_string();
    Ok(Value::String(Arc::new(id)))
}

// ─── typed uuid (arc 207 slice 2) ───────────────────────────────────────

/// Returns `true` iff `s` is a canonical 8-4-4-4-12 lowercase hyphenated UUID.
///
/// Exactly matches what `uuid::Uuid::to_string()` produces (and what the
/// `wat-edn` parser + `Uuid/to-string` emit). Rejects:
/// - Uppercase hex chars
/// - URN prefix (`urn:uuid:`)
/// - Braced form (`{...}`)
/// - Simple 32-char hex (no hyphens)
/// - Any other non-canonical variant
///
/// Used by `eval_uuid_typed_from_string` to enforce parse strictness.
fn is_canonical_uuid_string(s: &str) -> bool {
    s.len() == 36
        && s.as_bytes()[8] == b'-'
        && s.as_bytes()[13] == b'-'
        && s.as_bytes()[18] == b'-'
        && s.as_bytes()[23] == b'-'
        && s.chars().enumerate().all(|(i, c)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                c == '-'
            } else {
                c.is_ascii_hexdigit() && (!c.is_alphabetic() || c.is_ascii_lowercase())
            }
        })
}

/// `(:wat::core::Uuid/v4)` → `:wat::core::Uuid`.
///
/// Mints a fresh v4 (random) UUID on every call. Returns a typed
/// `:wat::core::Uuid` value — NOT a string. Arc 207 slice 2.
pub fn eval_uuid_typed_v4(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/v4";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
            span: args[0].span().clone(),
        });
    }
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v4()))
}

/// `(:wat::core::Uuid/v5 ns name)` → `:wat::core::Uuid`.
///
/// Deterministic SHA-1-based UUID. `ns` is `:wat::core::Uuid` (type-enforced,
/// eliminating the runtime-panic foot-gun in arc 206's string-typed namespace).
/// Returns a typed `:wat::core::Uuid`. Arc 207 slice 2.
pub fn eval_uuid_typed_v5(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/v5";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
            span: if args.is_empty() {
                crate::span::Span::unknown()
            } else {
                args[0].span().clone()
            },
        });
    }
    let ns_val = eval(&args[0], env, sym)?;
    let name_val = eval(&args[1], env, sym)?;
    let ns_uuid = match &ns_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid".into(),
                got: ns_val.type_name().into(),
                span: args[0].span().clone(),
            });
        }
    };
    let name_str = match &name_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String".into(),
                got: name_val.type_name().into(),
                span: args[1].span().clone(),
            });
        }
    };
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v5(ns_uuid, &name_str)))
}

/// `(:wat::core::Uuid/from-string s)` → `:Option<:wat::core::Uuid>`.
///
/// Parse-safe constructor. Accepts ONLY canonical 8-4-4-4-12 lowercase
/// hyphenated form; returns `None` for uppercase, URN prefix, braced,
/// simple (no-hyphen), or otherwise non-canonical input. Arc 207 slice 2.
pub fn eval_uuid_typed_from_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/from-string";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: if args.is_empty() {
                crate::span::Span::unknown()
            } else {
                args[0].span().clone()
            },
        });
    }
    let s_val = eval(&args[0], env, sym)?;
    let s = match &s_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String".into(),
                got: s_val.type_name().into(),
                span: args[0].span().clone(),
            });
        }
    };
    let result = if is_canonical_uuid_string(&s) {
        uuid::Uuid::parse_str(&s).ok().map(Value::wat__core__Uuid)
    } else {
        None
    };
    Ok(Value::Option(Arc::new(result)))
}

/// `(:wat::core::Uuid/to-string u)` → `:wat::core::String`.
///
/// Renders as canonical 8-4-4-4-12 lowercase hyphenated form. Arc 207 slice 2.
pub fn eval_uuid_typed_to_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/to-string";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
            span: if args.is_empty() {
                crate::span::Span::unknown()
            } else {
                args[0].span().clone()
            },
        });
    }
    let u_val = eval(&args[0], env, sym)?;
    let u = match &u_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid".into(),
                got: u_val.type_name().into(),
                span: args[0].span().clone(),
            });
        }
    };
    Ok(Value::String(Arc::new(u.to_string())))
}

/// `(:wat::core::Uuid/nil)` → `:wat::core::Uuid`.
///
/// Returns the nil UUID (`00000000-0000-0000-0000-000000000000`). Arc 207 slice 2.
pub fn eval_uuid_typed_nil(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/nil";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
            span: args[0].span().clone(),
        });
    }
    Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))
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
        span: args[0].span().clone(),
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
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(crate::span::Span::unknown);
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
            span,
        });
    }
    match eval(&args[0], env, sym)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: other.type_name(),
            span: args[0].span().clone(),
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
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(crate::span::Span::unknown);
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len(),
            span,
        });
    }
    let a = match eval(&args[0], env, sym)? {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: other.type_name(),
                span: args[0].span().clone(),
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
                span: args[1].span().clone(),
            });
        }
    };
    Ok((a, b))
}

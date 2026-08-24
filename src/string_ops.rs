//! `:wat::core::string::*` + `:wat::core::regex::*` + `:wat::core::Uuid/*`
//! — string basics, regex, and typed UUID primitives.
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
//! UUID — arc 207 slice 2 minted `:wat::core::Uuid` as a typed primitive.
//! Five verbs live here: `Uuid/v4` (random), `Uuid/v5` (deterministic SHA-1),
//! `Uuid/from-string` (parse-safe), `Uuid/to-string` (render), `Uuid/nil`
//! (zero-UUID sentinel). Arc 206's String-returning namespace-form verbs
//! (`:wat::core::uuid::v4` + `v5`) were retired in arc 207 slice 3.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use std::sync::Arc;

/// `(:wat::core::string::contains? haystack needle)` → `:bool`.
pub fn eval_string_contains(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, needle) = two_strings(":wat::core::string::contains?", args, env, sym, list_span)?;
    Ok(Value::bool(hay.contains(needle.as_str())))
}

/// `(:wat::core::string::starts-with? haystack prefix)` → `:bool`.
pub fn eval_string_starts_with(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, prefix) = two_strings(":wat::core::string::starts-with?", args, env, sym, list_span)?;
    Ok(Value::bool(hay.starts_with(prefix.as_str())))
}

/// `(:wat::core::string::ends-with? haystack suffix)` → `:bool`.
pub fn eval_string_ends_with(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let (hay, suffix) = two_strings(":wat::core::string::ends-with?", args, env, sym, list_span)?;
    Ok(Value::bool(hay.ends_with(suffix.as_str())))
}

/// `(:wat::core::string::length s)` → `:i64`.
///
/// Unicode scalar count — matches the user's mental model of "string
/// length" for scripts that use grapheme-sized characters. For byte
/// length, encode through `:wat::core::Vector<u8>` and use that vec's `length`.
pub fn eval_string_length(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::length", args, env, sym, list_span)?;
    Ok(Value::i64(s.chars().count() as i64))
}

/// `(:wat::core::string::trim s)` → `:String`.
pub fn eval_string_trim(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::trim", args, env, sym, list_span)?;
    Ok(Value::String(Arc::new(s.trim().to_string())))
}

/// `(:wat::core::string::to-lowercase s)` → `:String`.
///
/// Converts all ASCII/Unicode characters in `s` to their lowercase equivalent.
/// Pure and total (Rust `String::to_lowercase` is deterministic, no IO).
/// Arc 209 Stone C.3 — needed by `defservice` macro to derive fn names from PascalCase op keywords.
pub fn eval_string_to_lowercase(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::to-lowercase", args, env, sym, list_span)?;
    Ok(Value::String(Arc::new(s.to_lowercase())))
}

/// `(:wat::core::string::to-uppercase s)` → `:String`.
///
/// Converts all ASCII/Unicode characters in `s` to their uppercase equivalent.
/// Pure and total (Rust `String::to_uppercase` is deterministic, no IO).
/// Arc 209 naming-conversion stone — sibling of `to-lowercase`; needed by the
/// `kebab->pascal` wat helper to capitalize each segment's first character.
/// NOT on `is_pure_total` (no macro calls it; add only if a future macro needs it).
pub fn eval_string_to_uppercase(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::to-uppercase", args, env, sym, list_span)?;
    Ok(Value::String(Arc::new(s.to_uppercase())))
}

/// `(:wat::core::string::pascal->kebab s)` → `:String`.
///
/// PascalCase → kebab-case. Inserts a `-` before each uppercase character that
/// is NOT at position 0, then lowercases every character. Digits ride the current
/// word. Examples: `GetObject` → `get-object`, `Get` → `get`, `GetV2` → `get-v2`.
///
/// Pure and total on the disciplined subset (one uppercase letter per word, no
/// consecutive-capital acronym runs). On `is_pure_total` — the `defservice` macro
/// calls it at expand time to derive fn names from PascalCase op keywords.
/// Arc 209 naming-conversion stone.
pub fn eval_string_pascal_to_kebab(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let s = one_string(":wat::core::string::pascal->kebab", args, env, sym, list_span)?;
    let mut result = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            result.push('-');
        }
        for lc in ch.to_lowercase() {
            result.push(lc);
        }
    }
    Ok(Value::String(Arc::new(result)))
}

/// Resolve a keyword argument (used as a namespace) to the registry key form.
///
/// The registry key is the full keyword string with leading colon (e.g. `":my::aws"`).
/// Two value forms are accepted:
/// - `Value::wat__core__keyword(k)` — a keyword in value position (e.g. `:my::aws` literal).
///   The value already carries the leading colon.
/// - `Value::wat__WatAST(WatAST::Keyword(k, _))` — a WatAST keyword node bound as a
///   macro argument (type `:wat::WatAST`). The AST form also carries the leading colon.
///   (Mirrors the dual-arm handling in `keyword/to-string`.)
fn keyword_value_to_registry_key(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    let v = eval(arg, env, sym)?.value_owned();
    match v {
        Value::wat__core__keyword(ref k) => Ok((**k).clone()),
        Value::wat__WatAST(ref ast) => {
            if let WatAST::Keyword(k, _) = ast.as_ref() {
                Ok(k.clone())
            } else {
                Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: "keyword",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&v))
                }))
            }
        }
        ref other => Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "keyword",
            got: Box::new(crate::runtime::ValueSnapshot::of(other))
        })),
    }
}

/// `(:wat::core::string::pascal->kebab-in ns s)` → `:String`.
///
/// Namespace-scoped PascalCase → kebab-case. Reads `sym.acronym_registry[ns]`;
/// a registered acronym is ONE segment (e.g. `"ACL"` → one token `"acl"`);
/// capital-boundary for the rest. No entry for `ns` → plain `pascal->kebab`
/// behavior. Examples (with `["ACL"]` declared for `ns`):
///   `"CreateWebACL"` → `"create-web-acl"`, `"ACLRule"` → `"acl-rule"`.
///
/// On `is_pure_total` — the `defservice` macro calls it at expand time to
/// derive fn names from PascalCase op keywords using the namespace's declared
/// acronyms. Arc 265 acronym-registry stone.
pub fn eval_string_pascal_to_kebab_in(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::pascal->kebab-in";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        }));
    }
    let ns = keyword_value_to_registry_key(OP, &args[0], env, sym)?;
    let s = match eval(&args[1], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    };

    // Look up the acronym set for this namespace.
    let acronyms: &[String] = sym.acronym_registry.get(&ns).map(|v| v.as_slice()).unwrap_or(&[]);

    let result = pascal_to_kebab_with_acronyms(&s, acronyms);
    Ok(Value::String(Arc::new(result)))
}

/// Core algorithm: PascalCase → kebab-case using a known acronym set.
///
/// A registered acronym is ONE segment (e.g. `"ACL"` starts at an uppercase
/// boundary and consumes all its chars as a single token). Capital-boundary
/// for everything else. Empty acronym set → plain `pascal->kebab` behavior.
fn pascal_to_kebab_with_acronyms(s: &str, acronyms: &[String]) -> String {
    // Collect chars to allow look-ahead.
    let chars: Vec<char> = s.chars().collect();
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if i > 0 && ch.is_uppercase() {
            // Try to match a known acronym at this position.
            let mut matched: Option<usize> = None;
            for acr in acronyms {
                let acr_chars: Vec<char> = acr.chars().collect();
                if chars[i..].len() >= acr_chars.len() {
                    // Case-sensitive match (acronyms are canonical uppercase).
                    if chars[i..i + acr_chars.len()].iter().zip(&acr_chars).all(|(a, b)| a == b) {
                        // Ensure it's a word boundary — next char (if any) must be uppercase
                        // or end of string (not a lowercase continuation).
                        let end = i + acr_chars.len();
                        let at_boundary = end >= chars.len() || chars[end].is_uppercase();
                        if at_boundary {
                            matched = Some(acr_chars.len());
                            break;
                        }
                    }
                }
            }
            if let Some(len) = matched {
                // Flush current segment, then emit the whole acronym as one segment.
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                let acr_str: String = chars[i..i + len].iter().collect();
                segments.push(acr_str.to_lowercase());
                i += len;
                continue;
            } else {
                // Plain capital boundary — flush current.
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
        }
        // Accumulate lowercase.
        for lc in ch.to_lowercase() {
            current.push(lc);
        }
        i += 1;
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments.join("-")
}

/// `(:wat::core::string::kebab->pascal-in ns s)` → `:String`.
///
/// Namespace-scoped kebab-case → PascalCase. Reads `sym.acronym_registry[ns]`;
/// each segment matching a declared acronym (case-insensitive) is replaced by
/// the canonical form (e.g. `"acl"` → `"ACL"`); else capitalize (first char
/// upper, rest as-is). No entry for `ns` → plain `kebab->pascal` behavior.
/// Examples (with `["ACL"]` declared for `ns`):
///   `"create-web-acl"` → `"CreateWebACL"`.
///
/// NOT on `is_pure_total` — no macro needs it (only `pascal->kebab-in` is
/// called at defservice expand time). Arc 265 acronym-registry stone.
pub fn eval_string_kebab_to_pascal_in(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::kebab->pascal-in";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        }));
    }
    let ns = keyword_value_to_registry_key(OP, &args[0], env, sym)?;
    let s = match eval(&args[1], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    };

    // Look up the acronym set for this namespace.
    let acronyms: &[String] = sym.acronym_registry.get(&ns).map(|v| v.as_slice()).unwrap_or(&[]);

    let result = kebab_to_pascal_with_acronyms(&s, acronyms);
    Ok(Value::String(Arc::new(result)))
}

/// Core algorithm: kebab-case → PascalCase using a known acronym set.
///
/// Splits on `-`; each segment matching a registered acronym (case-insensitive)
/// → the canonical form (e.g. `"acl"` → `"ACL"`); else capitalize (first char
/// upper, rest as-is). Empty acronym set → plain `kebab->pascal` behavior.
pub(crate) fn kebab_to_pascal_with_acronyms(s: &str, acronyms: &[String]) -> String {
    let mut result = String::with_capacity(s.len());
    for segment in s.split('-') {
        // Check if this segment (case-insensitive) matches a known acronym.
        let canonical = acronyms.iter().find(|acr| acr.eq_ignore_ascii_case(segment));
        if let Some(acr) = canonical {
            result.push_str(acr);
        } else {
            // Plain capitalize: first char upper, rest as-is.
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                for uc in first.to_uppercase() {
                    result.push(uc);
                }
                result.push_str(chars.as_str());
            }
        }
    }
    result
}

/// `(:wat::core::string::subs s start end)` → `:String`.
///
/// Clojure's `subs`: start-inclusive, end-exclusive, CHAR-indexed.
/// `(subs "hello world" 0 5)` → `"hello"`.
/// `(subs "abc" 1 1)` → `""` (empty range).
/// Returns a clean RuntimeError (MalformedForm) for out-of-range indices.
pub fn eval_string_subs(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::subs";
    if args.len() != 3 {
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(|| list_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len()
        }));
    }
    let s = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    };
    let start = match eval(&args[1], env, sym)?.value_owned() {
        Value::i64(n) => n,
        other => return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "i64",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    };
    let end = match eval(&args[2], env, sym)?.value_owned() {
        Value::i64(n) => n,
        other => return Err(RuntimeError::new(args[2].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "i64",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    };
    let char_len = s.chars().count() as i64;
    if start < 0 || end < 0 || start > end || end > char_len {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "index out of range: start={start}, end={end}, char-length={char_len}; \
                 require 0 <= start <= end <= char-length"
            )
        }));
    }
    let result: String = s
        .chars()
        .skip(start as usize)
        .take((end - start) as usize)
        .collect();
    Ok(Value::String(Arc::new(result)))
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::split";
    let (hay, sep) = two_strings(OP, args, env, sym, list_span)?;
    if sep.is_empty() {
        return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "separator must not be empty".into()
        }));
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::join";
    if args.len() != 2 {
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(|| list_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        }));
    }
    let sep = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    let types = sym.types().map(|a| a.as_ref());
    // Converts the EvalBreak that `seqable_value_to_stream`/`crate::stream::realize`
    // raise (they live on the `eval_inner` signal subgraph) back to the plain
    // `RuntimeError` this fn returns — the same unwrap `eval`'s own public boundary
    // performs (runtime.rs's `eval`, `Err(EvalBreak::Signal(s)) => ...`); a `Signal`
    // escaping here is an interpreter bug, not a user-facing condition.
    let to_runtime_error = |span: &Span, e: EvalBreak| -> RuntimeError {
        match e {
            EvalBreak::Diagnostic(boxed) => *boxed,
            EvalBreak::Signal(s) => RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("internal: eval-loop signal escaped string::join's Seqable walk: {s}"),
            }),
        }
    };
    let pieces_owned: Vec<String> = match eval(&args[1], env, sym)?.value_owned() {
        // FAST PATH — unchanged (118.B7 discipline, Stone D contract): an eager
        // Vector keeps its direct iterator and never routes through the stream
        // normaliser below.
        Value::Vec(items) => items.iter().map(|item| render_str_total(item, types)).collect(),
        // WIDENED (Stone D, arc 255) — any other member of the `Seqable :- [T]`
        // surface (PersistentVector, List, Stream): normalise once through the
        // shared value-level door (`seqable_value_to_stream` — composes, does not
        // re-derive the container classification), then render each element as
        // the walk forces it. Single pass; nothing intermediate materialized.
        other => {
            let mut cur = crate::collection::transform::seqable_value_to_stream(other, OP, args[1].span())
                .map_err(|e| to_runtime_error(args[1].span(), e))?;
            let mut out = Vec::new();
            loop {
                let realized = crate::stream::realize(&cur, sym, args[1].span())
                    .map_err(|e| to_runtime_error(args[1].span(), e))?;
                match realized.as_ref() {
                    crate::stream::Stream::Empty => break,
                    crate::stream::Stream::Cons { head, tail } => {
                        out.push(render_str_total(head, types));
                        cur = Arc::clone(tail);
                    }
                    crate::stream::Stream::Thunk(_) | crate::stream::Stream::NativeThunk(_) => {
                        unreachable!("crate::stream::realize always returns Empty|Cons")
                    }
                }
            }
            out
        }
    };
    Ok(Value::String(Arc::new(pieces_owned.join(&sep))))
}

/// `:wat::core::str`'s rendering, factored so `str` and `join` cannot drift
/// (279.3). Total over every `Value`: two arms, both load-bearing.
///
/// - `Value::String` → itself, BARE (no surrounding quotes). The EDN encoder
///   quotes strings; routing a top-level `String` through it would corrupt
///   every caller that expects unquoted text (`(join "-" ["a" "b"])` would
///   render `"\"a\"-\"b\""` instead of `"a-b"`).
/// - everything else → `value_to_edn_string_with`, passed `types` so a
///   record renders by field NAME (`{:x 1}`) rather than positionally
///   (`{:field-0 1}`) — the 296/279.2 fix. Callers with no registry pass
///   `None` explicitly, per `edn_shim.rs:3490`'s stated discipline.
pub(crate) fn render_str_total(v: &Value, types: Option<&crate::types::TypeEnv>) -> String {
    match v {
        Value::String(s) => (**s).clone(),
        other => crate::edn_shim::value_to_edn_string_with(other, types),
    }
}

/// `(:wat::core::string::interpolate tmpl :k1 v1 :k2 v2 …)` → `:String`.
///
/// Pure-total runtime interpolation intrinsic. Same `{name}` + trailing `:name val`
/// kwargs grammar and `{{`/`}}` escape as the `format` macro (arc 279), but
/// interpolates at CALL time (not expand time) — making it **expand-time-legal**
/// (usable inside defmacro bodies where `format` is refused by the purity gate).
///
/// Strict: every `{name}` must have a matching `:name` kwarg (else RuntimeError);
/// every `:name` must be consumed (else RuntimeError). Repeated `{name}` against
/// one `:name` is fine. Lone `{`/`}` in the template is a RuntimeError.
/// Arc 284.
pub fn eval_string_interpolate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::interpolate";

    // Need at least the template arg.
    if args.is_empty() {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: 0,
        }));
    }

    // arg[0]: template — must eval to String.
    let tmpl = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
        })),
    };

    // args[1..]: must be an even count of (keyword, value) pairs.
    let rest = &args[1..];
    if !rest.len().is_multiple_of(2) {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "trailing kwargs must be :name value pairs — odd count".into(),
        }));
    }

    // Build name→rendered map and track which keys were used.
    let mut kwargs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < rest.len() {
        let key_arg = &rest[i];
        let val_arg = &rest[i + 1];
        // Key must be a keyword; eval it to get the keyword value.
        let key_name = match eval(key_arg, env, sym)?.value_owned() {
            Value::wat__core__keyword(k) => {
                // Strip the leading ':' to get the placeholder name.
                k.strip_prefix(':')
                    .unwrap_or(k.as_str())
                    .to_string()
            }
            other => return Err(RuntimeError::new(key_arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword (e.g. :name)",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
            })),
        };
        // Value: eval then render unquoted, through the same total door `str`/`join` use.
        let rendered = render_str_total(
            &eval(val_arg, env, sym)?.value_owned(),
            sym.types().map(|a| a.as_ref()),
        );
        kwargs.insert(key_name, rendered);
        i += 2;
    }

    // Parse the template char-by-char (mirrors the format macro's state machine).
    //   mode: "text" | "name"
    //   pending: "none" | "open" | "close"
    let mut result = String::with_capacity(tmpl.len());
    let chars: Vec<char> = tmpl.chars().collect();
    let mut idx = 0;
    let mut mode_name = false; // false = text mode, true = name mode
    let mut pending_open = false;
    let mut pending_close = false;
    let mut name_buf = String::new();

    while idx < chars.len() {
        let c = chars[idx];
        if !mode_name {
            // text mode
            if pending_open {
                pending_open = false;
                if c == '{' {
                    // {{ → literal {
                    result.push('{');
                } else if c == '}' {
                    // {} → empty placeholder name — error
                    return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "empty placeholder {} in template".into(),
                    }));
                } else {
                    // { followed by name char → enter name mode
                    mode_name = true;
                    name_buf.clear();
                    name_buf.push(c);
                }
            } else if pending_close {
                pending_close = false;
                if c == '}' {
                    // }} → literal }
                    result.push('}');
                } else {
                    return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: "lone '}' in template — use '}}' for a literal brace".into(),
                    }));
                }
            } else {
                // no pending
                if c == '{' {
                    pending_open = true;
                } else if c == '}' {
                    pending_close = true;
                } else {
                    result.push(c);
                }
            }
        } else {
            // name mode
            if c == '}' {
                // end of placeholder: look up name_buf in kwargs
                let name = name_buf.clone();
                match kwargs.get(&name) {
                    Some(val) => {
                        result.push_str(val);
                        used.insert(name);
                    }
                    None => return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!("missing kwarg for placeholder {{{}}}", name),
                    })),
                }
                mode_name = false;
                name_buf.clear();
            } else if c == '{' {
                return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "'{' inside placeholder name — unclosed '{'?".into(),
                }));
            } else {
                name_buf.push(c);
            }
        }
        idx += 1;
    }

    // Finalize: check for dangling open/close pending or open name mode.
    if mode_name {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("unclosed placeholder '{{{}}}'", name_buf),
        }));
    }
    if pending_open {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "lone '{' at end of template — use '{{' for a literal brace".into(),
        }));
    }
    if pending_close {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: "lone '}' at end of template — use '}}' for a literal brace".into(),
        }));
    }

    // Strict: every kwarg must have been referenced.
    for key in kwargs.keys() {
        if !used.contains(key) {
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("unused kwarg :{}", key),
            }));
        }
    }

    Ok(Value::String(Arc::new(result)))
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::string::concat";
    if args.is_empty() {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: 0
        }));
    }
    let mut total = 0usize;
    let mut pieces: Vec<Arc<String>> = Vec::with_capacity(args.len());
    for arg in args {
        match eval(arg, env, sym)?.value_owned() {
            Value::String(s) => {
                total += s.len();
                pieces.push(s);
            }
            other => {
                return Err(RuntimeError::new(arg.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other))
                }));
            }
        }
    }
    let mut out = String::with_capacity(total);
    for p in &pieces {
        out.push_str(p);
    }
    Ok(Value::String(Arc::new(out)))
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
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: the sole error (arity) locates at the first extra arg's own span (`args[0].span()`)

    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/v4";
    if !args.is_empty() {
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len()
        }));
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/v5";
    if args.len() != 2 {
        return Err(RuntimeError::new(if args.is_empty() {
                list_span.clone()
            } else {
                args[0].span().clone()
            }, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        }));
    }
    let ns_val = eval(&args[0], env, sym)?.value_owned();
    let name_val = eval(&args[1], env, sym)?.value_owned();
    let ns_uuid = match &ns_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(crate::runtime::ValueSnapshot::of(&ns_val))
            }));
        }
    };
    let name_str = match &name_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&name_val))
            }));
        }
    };
    Ok(Value::wat__core__Uuid(wat_edn::new_uuid_v5(ns_uuid, &name_str)))
}

/// `(:wat::core::Uuid/from-string s)` → `(:Option :- [:wat::core::Uuid])`.
///
/// Parse-safe constructor. Accepts ONLY canonical 8-4-4-4-12 lowercase
/// hyphenated form; returns `None` for uppercase, URN prefix, braced,
/// simple (no-hyphen), or otherwise non-canonical input. Arc 207 slice 2.
pub fn eval_uuid_typed_from_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/from-string";
    if args.len() != 1 {
        return Err(RuntimeError::new(if args.is_empty() {
                list_span.clone()
            } else {
                args[0].span().clone()
            }, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        }));
    }
    let s_val = eval(&args[0], env, sym)?.value_owned();
    let s = match &s_val {
        Value::String(s) => s.as_str().to_string(),
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&s_val))
            }));
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/to-string";
    if args.len() != 1 {
        return Err(RuntimeError::new(if args.is_empty() {
                list_span.clone()
            } else {
                args[0].span().clone()
            }, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        }));
    }
    let u_val = eval(&args[0], env, sym)?.value_owned();
    let u = match &u_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(crate::runtime::ValueSnapshot::of(&u_val))
            }));
        }
    };
    Ok(Value::String(Arc::new(u.to_string())))
}

/// `(:wat::core::Uuid/version u)` → `:wat::core::i64`.
///
/// Returns the version nibble of the UUID as an integer (e.g. 4 for a v4 UUID).
/// Arc 299 slice 1.
pub fn eval_uuid_version(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/version";
    if args.len() != 1 {
        return Err(RuntimeError::new(if args.is_empty() {
                list_span.clone()
            } else {
                args[0].span().clone()
            }, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        }));
    }
    let u_val = eval(&args[0], env, sym)?.value_owned();
    let u = match &u_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(crate::runtime::ValueSnapshot::of(&u_val))
            }));
        }
    };
    Ok(Value::i64(u.get_version_num() as i64))
}

/// `(:wat::core::Uuid/rfc4122-variant? u)` → `:wat::core::bool`.
///
/// Returns `true` iff the UUID's variant nibble indicates RFC-4122 (variant
/// bits `10xx` — nibble ∈ {8,9,a,b}). Arc 299 slice 1.
pub fn eval_uuid_rfc4122_variant(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/rfc4122-variant?";
    if args.len() != 1 {
        return Err(RuntimeError::new(if args.is_empty() {
                list_span.clone()
            } else {
                args[0].span().clone()
            }, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        }));
    }
    let u_val = eval(&args[0], env, sym)?.value_owned();
    let u = match &u_val {
        Value::wat__core__Uuid(u) => *u,
        _ => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::Uuid",
                got: Box::new(crate::runtime::ValueSnapshot::of(&u_val))
            }));
        }
    };
    Ok(Value::bool(u.get_variant() == uuid::Variant::RFC4122))
}

/// `(:wat::core::Uuid/nil)` → `:wat::core::Uuid`.
///
/// Returns the nil UUID (`00000000-0000-0000-0000-000000000000`). Arc 207 slice 2.
pub fn eval_uuid_typed_nil(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: the sole error (arity) locates at the first extra arg's own span (`args[0].span()`)

    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::Uuid/nil";
    if !args.is_empty() {
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len()
        }));
    }
    Ok(Value::wat__core__Uuid(uuid::Uuid::nil()))
}

// ─── Char ────────────────────────────────────────────────────────────────

/// `(:wat::core::char/of s)` → `:wat::core::char`.
///
/// Constructs a typed `:wat::core::char` from a length-1 String.
/// BMP-only: codepoints above U+FFFF (supplementary-plane) are rejected
/// with a clear diagnostic, inheriting the Stone 218.6b discipline from
/// wat-edn's BMP-only strictness.
/// Stone 242.1 — renamed from :wat::core::Char/of to :wat::core::char/of
/// (scalar types lowercase per Doctrine 2).
///
/// Errors:
/// - ArityMismatch: not exactly 1 argument.
/// - TypeMismatch: argument is not a `:wat::core::String`.
/// - MalformedForm: string is empty (needs length-1 String).
/// - MalformedForm: string has length > 1 char.
/// - MalformedForm: the single char is a supplementary-plane codepoint.
///
/// Arc 220 slice 2.
pub fn eval_char_of(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    // Stone 242.1 — renamed from :wat::core::Char/of to :wat::core::char/of.
    const OP: &str = ":wat::core::char/of";
    if args.len() != 1 {
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(|| list_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        }));
    }
    let val = eval(&args[0], env, sym)?.value_owned();
    let s = match val {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    let mut chars = s.chars();
    let c = match chars.next() {
        None => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "expected a length-1 String; got empty string".into()
            }));
        }
        Some(c) => c,
    };
    if chars.next().is_some() {
        let len = s.chars().count();
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "expected a length-1 String; got length-{} string {:?}",
                len, s
            )
        }));
    }
    if (c as u32) > 0xFFFF {
        return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "supplementary-plane codepoint U+{:X} not supported; \
                 wat::core::char is BMP-only (U+0000–U+FFFF)",
                c as u32
            )
        }));
    }
    Ok(Value::wat__core__Char(c))
}

/// Arc 220 Stone 220.4 — `(:wat::core::List/of arg1 arg2 ...)` variadic constructor.
///
/// Evaluates each argument and pushes it to the back of a new `LinkedList<Value>`.
/// Returns `Value::wat__core__List(Arc::new(list))`. Zero args → empty list.
/// No arity restriction (variadic; 0 or more). Mirrors `eval_char_of` pattern but
/// is variadic rather than fixed-arity.
pub fn eval_list_of(
    args: &[WatAST],
    _list_span: &Span, // rune:lint(unused-span) — located elsewhere: no own error path; the only errors are `?`-propagated from the per-element `eval(arg, …)`, each carrying the arg's own span

    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let mut items = std::collections::LinkedList::new();
    for arg in args {
        items.push_back(crate::runtime::eval(arg, env, sym)?.value_owned());
    }
    Ok(Value::wat__core__List(std::sync::Arc::new(items)))
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
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::core::regex::matches?";
    let (pattern, haystack) = two_strings(OP, args, env, sym, list_span)?;
    let re = regex::Regex::new(pattern.as_str()).map_err(|e| RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("invalid regex: {}", e)
    }))?;
    Ok(Value::bool(re.is_match(haystack.as_str())))
}

// ─── helpers ─────────────────────────────────────────────────────────────

fn one_string(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<String, RuntimeError> {
    if args.len() != 1 {
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(|| list_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        }));
    }
    match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn two_strings(
    op: &str,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<(String, String), RuntimeError> {
    if args.len() != 2 {
        let span = args
            .first()
            .map(|a| a.span().clone())
            .unwrap_or_else(|| list_span.clone());
        return Err(RuntimeError::new(span, RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len()
        }));
    }
    let a = match eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    let b = match eval(&args[1], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    Ok((a, b))
}

//! `:wat::string::*` intrinsics — arc 255 home #4, phase 2 (the string
//! carve), REGISTRY half of a builder-amended two-home split.
//!
//! The 19 Rust-implemented `:wat::string::*` verbs, carved out of the
//! now-retired `src/string_ops.rs` into the `#[wat_intrinsic]` fixed-arg
//! form (255.1b-ii), same shape as `intrinsic/bytes.rs` (arc 255's first
//! home). Each handler carries a `///` preamble the macro parses; the
//! attribute sniffs arity, emits the arity-checking `NativeHandler` shim,
//! and `inventory::submit!`s the (fqdn → shim) pair — no explicit
//! `register()` call. This module stays `mod`-declared in `intrinsic/mod.rs`
//! so its submissions are linked.
//!
//! **Two homes, builder ruling mid-flight (arc 255 home #4 phase 2):** this
//! file is the REGISTRY home — dispatch shim + `///` preamble only. The
//! algorithms these handlers call (`pascal_to_kebab_with_acronyms`,
//! `kebab_to_pascal_with_acronyms`, `keyword_value_to_registry_key`,
//! `render_str_total`) live in `src/string/mod.rs`, the NAMESPACE home — a
//! peer to `src/collection/`, `src/channel/`, `src/stream/`, etc.
//! `string_ops.rs` was FOUR domains in one loose root file
//! (`:wat::string::*`, `:wat::core::Uuid/*`, `:wat::core::char/*`,
//! `:wat::core::regex::*`); it no longer exists. Uuid → `intrinsic/uuid.rs`,
//! char → `intrinsic/char.rs`, regex → `intrinsic/regex.rs` (each "own home,
//! same shape" as `bytes.rs` — self-contained, no separate namespace home,
//! since none of their algorithms are shared with a non-carved call site the
//! way the string helpers above are). Three more `:wat::string::*` verbs
//! (`capitalize`, `kebab->pascal`, `strip-leading-colon`) are wat `defn`s in
//! `wat/string.wat`, not Rust intrinsics at all; not this stone's concern.
//!
//! ## The hazard this stone exists to avoid
//!
//! `runtime.rs:5394` consults the registry BEFORE the literal `match` — a
//! verb registered here while its old match arm survives in `runtime.rs`
//! would pass every test silently (the registration wins; the arm is dead
//! code nothing exercises). All 19 old arms are DELETED as part of this
//! carve, not left behind.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

// ─── shared per-arg helpers ────────────────────────────────────────────────
//
// The old slice-based `one_string`/`two_strings` (string_ops.rs) did double
// duty: type-check EACH arg AND arity-check the whole call (`args.len() !=
// N`). The arity half is now the `#[wat_intrinsic]` shim's job (it runs
// BEFORE any handler below is called), so these per-arg helpers only do the
// type-check half, located at the OFFENDING ARG's own span — never the
// call's list_span, which is why every handler below can leave its own
// `span` context param unused (see each `rune:lint(unused-span)` note).

/// Eval `arg` and require `:wat::core::String`; TypeMismatch locates at
/// `arg`'s own span.
fn arg_string(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Arc<String>, EvalBreak> {
    match eval_inner(arg, env, sym)?.value_owned() {
        Value::String(s) => Ok(s),
        other => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Eval `arg` and require `:wat::core::i64`; TypeMismatch locates at `arg`'s
/// own span. Shared by `subs`'s two index params.
fn arg_i64(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<i64, EvalBreak> {
    match eval_inner(arg, env, sym)?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

// `keyword_value_to_registry_key` and `pascal_to_kebab_with_acronyms` are NOT
// here — builder amendment: this file is the REGISTRY home (dispatch shim +
// `///` preamble); the algorithms the handlers below call live in
// `src/string/mod.rs`, the NAMESPACE home, alongside their twin
// `kebab_to_pascal_with_acronyms` (which never moved here at all, since
// `types.rs`/`runtime.rs` call it too — see `src/string/mod.rs`'s doc).

// ─── the 19 verbs ──────────────────────────────────────────────────────────

/// `(:wat::string::contains? haystack needle)` → whether `needle` occurs
/// anywhere in `haystack`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     haystack :wat::core::String the string searched
/// @arg     needle   :wat::core::String the substring sought
/// @ret     :wat::core::bool true iff `needle` occurs anywhere in `haystack`
/// @example (:wat::string::contains? "hello world" "wor") #=> true
/// @see     :wat::string::starts-with?
#[wat_intrinsic(":wat::string::contains?")]
pub(crate) fn eval_string_contains(
    haystack: &WatAST,
    needle: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at the offending arg's own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::contains?";
    let hay = arg_string(OP, haystack, env, sym)?;
    let needle = arg_string(OP, needle, env, sym)?;
    Ok(Value::bool(hay.contains(needle.as_str())))
}

/// `(:wat::string::starts-with? haystack prefix)` → whether `haystack` begins
/// with `prefix`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     haystack :wat::core::String the string examined
/// @arg     prefix   :wat::core::String the prefix sought
/// @ret     :wat::core::bool true iff `haystack` begins with `prefix`
/// @example (:wat::string::starts-with? "hello" "he") #=> true
/// @see     :wat::string::ends-with?
#[wat_intrinsic(":wat::string::starts-with?")]
pub(crate) fn eval_string_starts_with(
    haystack: &WatAST,
    prefix: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at the offending arg's own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::starts-with?";
    let hay = arg_string(OP, haystack, env, sym)?;
    let prefix = arg_string(OP, prefix, env, sym)?;
    Ok(Value::bool(hay.starts_with(prefix.as_str())))
}

/// `(:wat::string::ends-with? haystack suffix)` → whether `haystack` ends
/// with `suffix`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     haystack :wat::core::String the string examined
/// @arg     suffix   :wat::core::String the suffix sought
/// @ret     :wat::core::bool true iff `haystack` ends with `suffix`
/// @example (:wat::string::ends-with? "hello" "lo") #=> true
/// @see     :wat::string::starts-with?
#[wat_intrinsic(":wat::string::ends-with?")]
pub(crate) fn eval_string_ends_with(
    haystack: &WatAST,
    suffix: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at the offending arg's own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::ends-with?";
    let hay = arg_string(OP, haystack, env, sym)?;
    let suffix = arg_string(OP, suffix, env, sym)?;
    Ok(Value::bool(hay.ends_with(suffix.as_str())))
}

/// `(:wat::string::length s)` → the number of Unicode scalar values in `s`.
///
/// Unicode scalar count (`chars().count()`) — matches the mental model of
/// "string length" for scripts using grapheme-sized characters, not UTF-8
/// byte length. For byte length, encode through `:wat::core::Vector<u8>`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to measure
/// @ret     :wat::core::i64 the number of Unicode scalar values in `s`
/// @example (:wat::string::length "hello") #=> 5
#[wat_intrinsic(":wat::string::length")]
pub(crate) fn eval_string_length(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::length", s, env, sym)?;
    Ok(Value::i64(s.chars().count() as i64))
}

/// `(:wat::string::trim s)` → `s` with leading and trailing whitespace
/// removed.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to trim
/// @ret     :wat::core::String the string with leading and trailing whitespace removed
/// @example (:wat::string::trim "  x  ") #=> "x"
/// @see     :wat::string::to-lowercase
#[wat_intrinsic(":wat::string::trim")]
pub(crate) fn eval_string_trim(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::trim", s, env, sym)?;
    Ok(Value::String(Arc::new(s.trim().to_string())))
}

/// `(:wat::string::to-lowercase s)` → `s` with every character lowercased.
///
/// Pure and total (Rust's `String::to_lowercase` is deterministic, no IO).
/// Arc 209 Stone C.3 — needed by the `defservice` macro to derive fn names
/// from PascalCase op keywords.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to lowercase
/// @ret     :wat::core::String `s` with every character lowercased
/// @example (:wat::string::to-lowercase "HELLO") #=> "hello"
/// @see     :wat::string::to-uppercase
#[wat_intrinsic(":wat::string::to-lowercase")]
pub(crate) fn eval_string_to_lowercase(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::to-lowercase", s, env, sym)?;
    Ok(Value::String(Arc::new(s.to_lowercase())))
}

/// `(:wat::string::to-uppercase s)` → `s` with every character uppercased.
///
/// Pure and total (Rust's `String::to_uppercase` is deterministic, no IO).
/// Arc 209 naming-conversion stone; needed by the `kebab->pascal` wat helper
/// to capitalize each segment's first character.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to uppercase
/// @ret     :wat::core::String `s` with every character uppercased
/// @example (:wat::string::to-uppercase "hello") #=> "HELLO"
/// @see     :wat::string::to-lowercase
#[wat_intrinsic(":wat::string::to-uppercase")]
pub(crate) fn eval_string_to_uppercase(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::to-uppercase", s, env, sym)?;
    Ok(Value::String(Arc::new(s.to_uppercase())))
}

/// `(:wat::string::pascal->kebab s)` → PascalCase `s` converted to
/// kebab-case.
///
/// Inserts a `-` before each uppercase character not at position 0, then
/// lowercases every character. Digits ride the current word. Examples:
/// `GetObject` → `get-object`, `Get` → `get`, `GetV2` → `get-v2`. Pure and
/// total on the disciplined subset (one uppercase letter per word, no
/// consecutive-capital acronym runs) — `pascal->kebab-in` handles acronyms.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the PascalCase string to convert
/// @ret     :wat::core::String the kebab-case rendering of `s`
/// @example (:wat::string::pascal->kebab "GetObject") #=> "get-object"
/// @see     :wat::string::pascal->kebab-in
#[wat_intrinsic(":wat::string::pascal->kebab")]
pub(crate) fn eval_string_pascal_to_kebab(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`)
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::pascal->kebab", s, env, sym)?;
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

/// `(:wat::string::pascal->kebab-in ns s)` → namespace-scoped PascalCase →
/// kebab-case.
///
/// Reads `sym.acronym_registry[ns]`; a registered acronym is ONE segment
/// (e.g. `"ACL"` → one token `"acl"`); capital-boundary for the rest. No
/// entry for `ns` → plain `pascal->kebab` behavior. Called by the
/// `defservice` macro at expand time to derive fn names from PascalCase op
/// keywords using the namespace's declared acronyms (arc 265).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     ns :wat::core::keyword the namespace whose declared acronyms apply
/// @arg     s  :wat::core::String  the PascalCase string to convert
/// @ret     :wat::core::String the kebab-case rendering of `s`
/// @example (:wat::string::pascal->kebab-in :my-ns "GetObject") #=> "get-object"
/// @see     :wat::string::kebab->pascal-in
#[wat_intrinsic(":wat::string::pascal->kebab-in")]
pub(crate) fn eval_string_pascal_to_kebab_in(
    ns: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error locates at its own arg's span (`ns`'s via `keyword_value_to_registry_key`, `s`'s via `arg_string`)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::pascal->kebab-in";
    let ns = crate::string::keyword_value_to_registry_key(OP, ns, env, sym)?;
    let s = arg_string(OP, s, env, sym)?;
    let acronyms: &[String] = sym.acronym_registry.get(&ns).map(|v| v.as_slice()).unwrap_or(&[]);
    let result = crate::string::pascal_to_kebab_with_acronyms(&s, acronyms);
    Ok(Value::String(Arc::new(result)))
}

/// `(:wat::string::kebab->pascal-in ns s)` → namespace-scoped kebab-case →
/// PascalCase.
///
/// Reads `sym.acronym_registry[ns]`; each segment matching a declared
/// acronym (case-insensitive) → the canonical form (e.g. `"acl"` → `"ACL"`);
/// else capitalize (first char upper, rest as-is). No entry for `ns` → plain
/// capitalize-every-segment behavior. Arc 265 acronym-registry stone.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     ns :wat::core::keyword the namespace whose declared acronyms apply
/// @arg     s  :wat::core::String  the kebab-case string to convert
/// @ret     :wat::core::String the PascalCase rendering of `s`
/// @example (:wat::string::kebab->pascal-in :my-ns "get-object") #=> "GetObject"
/// @see     :wat::string::pascal->kebab-in
#[wat_intrinsic(":wat::string::kebab->pascal-in")]
pub(crate) fn eval_string_kebab_to_pascal_in(
    ns: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error locates at its own arg's span (`ns`'s via `keyword_value_to_registry_key`, `s`'s via `arg_string`)
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::kebab->pascal-in";
    let ns = crate::string::keyword_value_to_registry_key(OP, ns, env, sym)?;
    let s = arg_string(OP, s, env, sym)?;
    let acronyms: &[String] = sym.acronym_registry.get(&ns).map(|v| v.as_slice()).unwrap_or(&[]);
    let result = crate::string::kebab_to_pascal_with_acronyms(&s, acronyms);
    Ok(Value::String(Arc::new(result)))
}

/// `(:wat::string::subs s start end)` → the CHAR-indexed substring
/// `[start, end)`.
///
/// Clojure's `subs`: start-inclusive, end-exclusive. `(subs "hello world" 0
/// 5)` → `"hello"`. `(subs "abc" 1 1)` → `""` (empty range). Out-of-range
/// indices raise a clean diagnostic rather than panicking.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s     :wat::core::String the string to slice
/// @arg     start :wat::core::i64    the start index, inclusive
/// @arg     end   :wat::core::i64    the end index, exclusive
/// @ret     :wat::core::String the substring `s[start..end)`, char-indexed
/// @example (:wat::string::subs "hello world" 0 5) #=> "hello"
#[wat_intrinsic(":wat::string::subs")]
pub(crate) fn eval_string_subs(
    s: &WatAST,
    start: &WatAST,
    end: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::subs";
    let s = arg_string(OP, s, env, sym)?;
    let start = arg_i64(OP, start, env, sym)?;
    let end = arg_i64(OP, end, env, sym)?;
    let char_len = s.chars().count() as i64;
    if start < 0 || end < 0 || start > end || end > char_len {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "index out of range: start={start}, end={end}, char-length={char_len}; \
                     require 0 <= start <= end <= char-length"
                ),
            },
        )
        .into());
    }
    let result: String = s.chars().skip(start as usize).take((end - start) as usize).collect();
    Ok(Value::String(Arc::new(result)))
}

/// `(:wat::string::split haystack sep)` → every piece of `haystack` between
/// occurrences of `sep`.
///
/// An empty `sep` — the edge case `str::split("")` would degenerate to
/// per-char — is refused as a MalformedForm: almost always a bug, never
/// obvious what the caller wanted. Callers who genuinely want per-char
/// iteration can encode through `Vec<u8>` via the IO layer.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     haystack :wat::core::String the string to split
/// @arg     sep      :wat::core::String the separator; must not be empty
/// @ret     (:wat::core::Vector :- [:wat::core::String]) the pieces of `haystack` between occurrences of `sep`
/// @example (:wat::string::split "a,b,c" ",") #=> (:wat::core::Vector :wat::core::String "a" "b" "c")
/// @see     :wat::string::join
#[wat_intrinsic(":wat::string::split")]
pub(crate) fn eval_string_split(
    haystack: &WatAST,
    sep: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error (TypeMismatch/MalformedForm) locates at `haystack`'s or `sep`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::split";
    let hay = arg_string(OP, haystack, env, sym)?;
    let sep_val = arg_string(OP, sep, env, sym)?;
    if sep_val.is_empty() {
        return Err(RuntimeError::new(
            sep.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "separator must not be empty".into(),
            },
        )
        .into());
    }
    let pieces: Vec<Value> =
        hay.split(sep_val.as_str()).map(|s| Value::String(Arc::new(s.to_string()))).collect();
    Ok(Value::Vec(Arc::new(pieces)))
}

/// `(:wat::string::join sep pieces)` → every element of `pieces`, rendered
/// and joined by `sep`.
///
/// Signature order matches Rust's `Vec::<String>::join(&sep)`: separator
/// first (the uniform thing), pieces second (the per-call thing). `pieces`
/// accepts the full `Seqable :- [T]` surface (Vector, PersistentVector,
/// List, Stream) — each element renders through the same total door `str`
/// uses (`render_str_total`, `string_ops.rs`, 279.3), so `join` and `str`
/// cannot drift.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     sep    :wat::core::String the separator
/// @arg     pieces (:wat::core::Seqable :- [T]) the elements to render and join
/// @ret     :wat::core::String every element of `pieces`, rendered and joined by `sep`
/// @example (:wat::string::join "-" (:wat::core::Vector :wat::core::String "a" "b")) #=> "a-b"
/// @see     :wat::string::split
#[wat_intrinsic(":wat::string::join")]
pub(crate) fn eval_string_join(
    sep: &WatAST,
    pieces: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error locates at `sep`'s or `pieces`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::join";
    let sep_val = arg_string(OP, sep, env, sym)?;
    let types = sym.types().map(|a| a.as_ref());
    // Converts the EvalBreak that `seqable_value_to_stream`/`crate::stream::realize`
    // raise back to a plain `RuntimeError`, mirroring `eval`'s own public-boundary
    // unwrap; a `Signal` escaping here is an interpreter bug, not a user-facing
    // condition.
    let to_runtime_error = |span: &Span, e: EvalBreak| -> RuntimeError {
        match e {
            EvalBreak::Diagnostic(boxed) => *boxed,
            EvalBreak::Signal(s) => RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("internal: eval-loop signal escaped string::join's Seqable walk: {s}"),
                },
            ),
        }
    };
    let pieces_owned: Vec<String> = match eval_inner(pieces, env, sym)?.value_owned() {
        // FAST PATH — an eager Vector keeps its direct iterator and never routes
        // through the stream normaliser below.
        Value::Vec(items) => {
            items.iter().map(|item| crate::string::render_str_total(item, types)).collect()
        }
        // WIDENED (Stone D, arc 255) — any other member of the `Seqable :- [T]`
        // surface: normalise once through the shared value-level door, then render
        // each element as the walk forces it.
        other => {
            let mut cur = crate::collection::transform::seqable_value_to_stream(other, OP, pieces.span())
                .map_err(|e| to_runtime_error(pieces.span(), e))?;
            let mut out = Vec::new();
            loop {
                let realized = crate::stream::realize(&cur, sym, pieces.span())
                    .map_err(|e| to_runtime_error(pieces.span(), e))?;
                match realized.as_ref() {
                    crate::stream::Stream::Empty => break,
                    crate::stream::Stream::Cons { head, tail } => {
                        out.push(crate::string::render_str_total(head, types));
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
    Ok(Value::String(Arc::new(pieces_owned.join(sep_val.as_str()))))
}

/// `(:wat::string::concat s1 s2 ... sn)` → every argument, concatenated
/// positionally.
///
/// Differs from `join` in that there's no separator and the args are passed
/// positionally rather than packed into a `Vector<String>` — the natural
/// form for "stitch a few strings together at the call site." Equivalent to
/// `(:wat::string::join "" (:wat::core::Vector :wat::core::String s1 s2
/// ...))` but spares the caller the Vec ceremony. Arity: 1+; the empty arg
/// list errors (the empty string has no useful concat semantics worth
/// special-casing).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     args… :wat::core::String the strings to concatenate, in order
/// @ret     :wat::core::String every argument, concatenated in order
/// @example (:wat::string::concat "a" "b" "c") #=> "abc"
/// @see     :wat::string::join
#[wat_intrinsic(":wat::string::concat")]
pub(crate) fn eval_string_concat(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::concat";
    if args.is_empty() {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 1, got: 0 },
        )
        .into());
    }
    let mut total = 0usize;
    let mut pieces: Vec<Arc<String>> = Vec::with_capacity(args.len());
    for arg in args {
        match eval_inner(arg, env, sym)?.value_owned() {
            Value::String(s) => {
                total += s.len();
                pieces.push(s);
            }
            other => {
                return Err(RuntimeError::new(
                    arg.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "String",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        }
    }
    let mut out = String::with_capacity(total);
    for p in &pieces {
        out.push_str(p);
    }
    Ok(Value::String(Arc::new(out)))
}

/// `(:wat::string::interpolate tmpl :k1 v1 :k2 v2 …)` → `tmpl` with each
/// `{name}` replaced by its rendered `:name` kwarg.
///
/// Pure-total runtime interpolation. Same `{name}` + trailing `:name val`
/// kwargs grammar and `{{`/`}}` escape as the `format` macro (arc 279), but
/// interpolates at CALL time (not expand time) — making it
/// **expand-time-legal** (usable inside defmacro bodies where `format` is
/// refused by the purity gate). Strict: every `{name}` must have a matching
/// `:name` kwarg (else RuntimeError); every `:name` must be consumed (else
/// RuntimeError). Repeated `{name}` against one `:name` is fine. A lone `{`
/// or `}` in the template is a RuntimeError. Arc 284.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     args… :wat::core::Value the template followed by `:name value` kwarg pairs
/// @ret     :wat::core::String `tmpl` with each `{name}` replaced by its rendered `:name` kwarg
/// @example (:wat::string::interpolate "hi {name}" :name "world") #=> "hi world"
#[wat_intrinsic(":wat::string::interpolate")]
pub(crate) fn eval_string_interpolate(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::string::interpolate";

    if args.is_empty() {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 1, got: 0 },
        )
        .into());
    }

    let tmpl = arg_string(OP, &args[0], env, sym)?;

    let rest = &args[1..];
    if !rest.len().is_multiple_of(2) {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "trailing kwargs must be :name value pairs — odd count".into(),
            },
        )
        .into());
    }

    let mut kwargs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < rest.len() {
        let key_arg = &rest[i];
        let val_arg = &rest[i + 1];
        let key_name = match eval_inner(key_arg, env, sym)?.value_owned() {
            Value::wat__core__keyword(k) => k.strip_prefix(':').unwrap_or(k.as_str()).to_string(),
            other => {
                return Err(RuntimeError::new(
                    key_arg.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "keyword (e.g. :name)",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        };
        let rendered = crate::string::render_str_total(
            &eval_inner(val_arg, env, sym)?.value_owned(),
            sym.types().map(|a| a.as_ref()),
        );
        kwargs.insert(key_name, rendered);
        i += 2;
    }

    let mut result = String::with_capacity(tmpl.len());
    let chars: Vec<char> = tmpl.chars().collect();
    let mut idx = 0;
    let mut mode_name = false;
    let mut pending_open = false;
    let mut pending_close = false;
    let mut name_buf = String::new();

    while idx < chars.len() {
        let c = chars[idx];
        if !mode_name {
            if pending_open {
                pending_open = false;
                if c == '{' {
                    result.push('{');
                } else if c == '}' {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "empty placeholder {} in template".into(),
                        },
                    )
                    .into());
                } else {
                    mode_name = true;
                    name_buf.clear();
                    name_buf.push(c);
                }
            } else if pending_close {
                pending_close = false;
                if c == '}' {
                    result.push('}');
                } else {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: "lone '}' in template — use '}}' for a literal brace".into(),
                        },
                    )
                    .into());
                }
            } else if c == '{' {
                pending_open = true;
            } else if c == '}' {
                pending_close = true;
            } else {
                result.push(c);
            }
        } else if c == '}' {
            let name = name_buf.clone();
            match kwargs.get(&name) {
                Some(val) => {
                    result.push_str(val);
                    used.insert(name);
                }
                None => {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: OP.into(),
                            reason: format!("missing kwarg for placeholder {{{}}}", name),
                        },
                    )
                    .into());
                }
            }
            mode_name = false;
            name_buf.clear();
        } else if c == '{' {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "'{' inside placeholder name — unclosed '{'?".into(),
                },
            )
            .into());
        } else {
            name_buf.push(c);
        }
        idx += 1;
    }

    if mode_name {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!("unclosed placeholder '{{{}}}'", name_buf),
            },
        )
        .into());
    }
    if pending_open {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "lone '{' at end of template — use '{{' for a literal brace".into(),
            },
        )
        .into());
    }
    if pending_close {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "lone '}' at end of template — use '}}' for a literal brace".into(),
            },
        )
        .into());
    }

    for key in kwargs.keys() {
        if !used.contains(key) {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("unused kwarg :{}", key),
                },
            )
            .into());
        }
    }

    Ok(Value::String(Arc::new(result)))
}

/// `(:wat::string::declare-acronyms ns acronyms)` → a runtime no-op.
///
/// The real work happens BEFORE eval reaches this handler:
/// `preregister_acronyms` (a pre-pass over the frozen residue, `runtime.rs`)
/// walks every `declare-acronyms` form and populates
/// `sym.acronym_registry[ns]` with `acronyms` — consumed later by
/// `pascal->kebab-in` / `kebab->pascal-in`, including at `defservice`
/// macro-expand time. By the time evaluation reaches THIS handler the
/// registry is already built, so both args are accepted and ignored. The
/// type-checker (`check.rs`'s own `:wat::string::declare-acronyms` arm, via
/// `parse_declare_acronyms_form`) already enforces the 2-arg `(:ns ["ACL"
/// ...])` shape before eval ever sees a form here — so the `#[wat_intrinsic]`
/// shim's `Exact(2)` arity check (new; the old inline arm never checked)
/// cannot reject anything a well-typed program would send it.
///
/// Marked `@Purity Pure`, not `Effectful`, even though the design's arc-265
/// intent for the VERB is a durable declaration: `intrinsic/mod.rs`'s own
/// `declared_purity_vs_effectful_by_prefix_census` test enforces `Effectful
/// ⇒ effectful_by_prefix(name)` for every REGISTERED row — a safety net
/// against a row that would read as falsely "safe" if ever looked up before
/// registration (`runtime.rs`'s `is_effectful_op` falls back to the
/// namespace-prefix guess for anything not yet in the registry).
/// `:wat::string::` is not one of the prefixes `effectful_by_prefix` treats
/// as effectful (`:wat::kernel::` / `:wat::io::` / `:wat::eval-` /
/// `:wat::load` / `:wat::config::`), and this is NOT this carve's stone to
/// widen that list. `Pure` is also the accurate, narrower claim about what
/// THIS handler's own body does, once the pre-pass model above is granted:
/// zero side effects at the point evaluation reaches it, unlike
/// `:wat::kernel::reset-sigusr1!` (`intrinsic/kernel/ambient.rs`), whose
/// `:wat::kernel::` prefix DOES earn the `Effectful` marking the census
/// checks for.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Declaration
/// @arg     _ns        :wat::core::keyword the namespace the acronyms are declared for (unused here — registered by the pre-pass)
/// @arg     _acronyms  :wat::core::Vector   the acronym literals (unused here — registered by the pre-pass)
/// @ret     :wat::core::nil always nil
/// @example (:wat::string::declare-acronyms :my-ns ["ACL"]) #=> nil
#[wat_intrinsic(":wat::string::declare-acronyms")]
pub(crate) fn eval_string_declare_acronyms(
    _ns: &WatAST,
    _acronyms: &WatAST,
    _env: &Environment, // rune:lint(unused-env) — both args are accepted and ignored; see doc block above
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    _span: &Span, // rune:lint(unused-span) — infallible — no error path (always `Ok(Value::Unit)`)
) -> Result<Value, EvalBreak> {
    Ok(Value::Unit)
}

/// `(:wat::string::to-i64 s)` → `s` parsed as a base-10 `:i64`, or `None` if
/// it does not parse.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to parse
/// @ret     (:wat::core::Option :- [:wat::core::i64]) `Some(n)` on a valid base-10 i64 literal, `None` otherwise
/// @example (:wat::string::to-i64 "42") #=> (:wat::core::Some 42)
/// @example (:wat::string::to-i64 "nope") #=> :None
/// @see     :wat::string::to-f64
#[wat_intrinsic(":wat::string::to-i64")]
pub(crate) fn eval_string_to_i64(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`); an unparseable string is a non-error `Ok(None)`
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::to-i64", s, env, sym)?;
    let parsed = s.parse::<i64>().ok().map(Value::i64);
    Ok(Value::Option(Arc::new(parsed)))
}

/// `(:wat::string::to-f64 s)` → `s` parsed as an `:f64`, or `None` if it does
/// not parse.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to parse
/// @ret     (:wat::core::Option :- [:wat::core::f64]) `Some(x)` on a valid f64 literal, `None` otherwise
/// @example (:wat::string::to-f64 "3.5") #=> (:wat::core::Some 3.5)
/// @example (:wat::string::to-f64 "nope") #=> :None
/// @see     :wat::string::to-bool
#[wat_intrinsic(":wat::string::to-f64")]
pub(crate) fn eval_string_to_f64(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`); an unparseable string is a non-error `Ok(None)`
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::to-f64", s, env, sym)?;
    let parsed = s.parse::<f64>().ok().map(Value::f64);
    Ok(Value::Option(Arc::new(parsed)))
}

/// `(:wat::string::to-bool s)` → `Some(true)` for `"true"`, `Some(false)` for
/// `"false"`, `None` otherwise.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the string to parse
/// @ret     (:wat::core::Option :- [:wat::core::bool]) `Some(b)` for exactly `"true"`/`"false"`, `None` otherwise
/// @example (:wat::string::to-bool "true") #=> (:wat::core::Some true)
/// @example (:wat::string::to-bool "nope") #=> :None
/// @see     :wat::string::to-i64
#[wat_intrinsic(":wat::string::to-bool")]
pub(crate) fn eval_string_to_bool(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the only error (TypeMismatch) locates at `s`'s own span (`arg_string`); an unrecognized string is a non-error `Ok(None)`
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::string::to-bool", s, env, sym)?;
    let parsed = match s.as_str() {
        "true" => Some(Value::bool(true)),
        "false" => Some(Value::bool(false)),
        _ => None,
    };
    Ok(Value::Option(Arc::new(parsed)))
}

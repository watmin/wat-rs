//! wat — the wat language frontend + runtime.
//!
//! Self-reference: `extern crate self as wat;` makes the crate
//! accessible by its own name from within. The `#[wat_dispatch]`
//! macro emits `::wat::...` paths in its generated code, which
//! resolve identically whether the macro is invoked inside this
//! crate or from a downstream consumer.
//!
//! This crate implements the wat language as specified by the 058 algebra
//! surface proposal batch in the holon-lab-trading repo. It depends on
//! `holon` (holon-rs) for the algebra substrate — the 6 core forms
//! (`Atom`, `Bind`, `Bundle`, `Permute`, `Thermometer`, `Blend`), the
//! measurements tier (`cosine`, `dot`), and the atom type registry.
//!
//! # Modules
//!
//! - [`ast`] — `WatAST`, the language-surface AST the parser produces.
//!   Distinct from `wat::holon::HolonAST` — the WatAST carries `define`,
//!   `fn`, `struct`, `enum`, `newtype`, `typealias`, `load!`, `set-*!`,
//!   `let`, `if`, `match`, `defmacro`, and all the language-level forms.
//!   Algebra-core calls appear as `UpperCall` nodes that are lowered to
//!   `HolonAST` at evaluation time.
//! - [`lexer`] — tokenizer for the s-expression surface. Handles
//!   keyword-path tokens, the colon-quoting rule, string/numeric/bool
//!   literals, comments.
//! - [`parser`] — tokens → `WatAST`.
//! - [`config`] — entry-file discipline + `set-*!` setter commit.
//! - [`load`] — recursive load-form resolution. Six load forms
//!   (`load-file!` / `load-string!` / `digest-load!` / `digest-load-string!` /
//!   `signed-load!` / `signed-load-string!`) — each takes its source
//!   directly; verification payloads use `:wat::verify::*` keywords
//!   (arc 028 iface drop).
//! - [`scope`] — `Identifier` with `BTreeSet<ScopeId>` scope sets
//!   for Racket sets-of-scopes hygiene.
//! - [`macros`] — `defmacro` with quasiquote + hygiene.
//! - [`types`] — type declarations (`struct`, `enum`, `newtype`,
//!   `typealias`) + `TypeEnv`.
//! - [`resolve`] — post-expansion name resolution over the symbol
//!   table and type environment.
//! - [`check`] — rank-1 Hindley-Milner type check (slice 7b). Real
//!   parametric polymorphism, substitution, instantiation; `:Any` is
//!   banned per 058-030.
//! - [`hash`] — canonical-EDN serialization + SHA-256 hashing +
//!   Ed25519 signature verification.
//! - [`lower`] — `WatAST` algebra-core subtree → `wat::holon::HolonAST`.
//! - [`runtime`] — AST-walker for `define` / `fn` / `let` / `if`
//!   + algebra-core dispatch.
//! - [`freeze`] — the 12-step startup pipeline that composes parse →
//!   resolve → check → freeze into a single world.
//! - [`rust_deps`] — `:rust::*` namespace registry + marshaling
//!   traits (`FromWat` / `ToWat`) + `ThreadOwnedCell<T>` /
//!   `OwnedMoveCell<T>` scope primitives.
//! - [`distribution`] — `run` / `Battery`: published surface for
//!   third-party wat distributions (arc 170, folded in from the
//!   former `wat-cli` crate). Backs the `wat` / `cargo-wat` binaries
//!   under `src/bin/`.
//! - [`stdlib`] — baked-in wat source files (Subtract, Console,
//!   LocalCache, Cache, …) registered before user code parses.

extern crate self as wat;

pub mod assertion;
pub mod ast;
pub mod check;
pub mod closure_extract;
pub(crate) mod collection;
pub(crate) mod rete;
pub mod argspec;
pub(crate) mod remedy;
pub mod capability;
pub mod comms;
pub mod kernel;
pub mod config;
pub mod distribution;
pub mod edn;
pub mod error_ns;
pub mod sigma;
pub mod process;
pub mod form_match;
pub mod freeze;
pub(crate) mod function;
pub mod hash;
pub mod hologram;
pub mod scope;
pub mod io;
pub mod lexer;
pub mod load;
pub mod lower;
pub mod macros;
pub mod parser;
pub mod resolve;
pub mod restriction_entry;
pub mod runtime;
pub mod stream;
pub mod rust_deps;
pub mod panic_hook;
pub mod span;
pub mod services;
pub mod special_forms;
pub mod string;
pub mod host;
pub use services::{
    install_thread_io, uninstall_thread_io,
    ThreadIO,
};
pub mod channel;
pub mod time;
pub mod types;
pub mod value;
pub mod vm_registry;
pub(crate) mod intrinsic;

pub use host::compose::{compose_and_run, compose_and_run_with_loader};
pub use load::source::WatSource;
pub use span::Span;
pub use wat_macros::{main, test};

pub use ast::WatAST;
pub use check::{check_program, CheckEnv, CheckError, CheckErrorKind, CheckErrors, TypeScheme};
// Arc 093 — bridge EDN text to a runtime Value using the type
// registry. `read_edn` is parse + bridge in one call;
// `edn_to_value` operates on an already-parsed EDN tree.
//
// Arc 294.j — arc-093's standalone tagged-HolonAST-from-EDN readers (the pair
// this export line used to carry) are GONE, not merely unexported: their tag
// family died on the write side (DESIGN-STONE-294.j), and a
// `:wat::holon::HolonAST`-typed decode now goes through the ordinary
// `edn_to_value` / `edn_to_typed_value` path like any other type. Measured
// zero callers in this repo AND in every sibling repo under `holon/`
// (STOP-3's required check) before deleting.
pub use edn::render::{edn_to_value, read_edn, EdnReadError};
pub use config::{
    collect_entry_file, collect_entry_file_with_inherit, CapacityMode, Config, ConfigError,
};
pub use sigma::{DefaultCoincidentSigma, DefaultPresenceSigma, SigmaFn, WatFnSigmaFn};
pub use vm_registry::{Encoders, EncoderRegistry};
pub use freeze::{
    bootstrap_wat_vm_process, eval_digest_in_frozen, eval_in_frozen, eval_signed_in_frozen,
    invoke_user_main, invoke_user_main_with_program, resolve_env_program, startup_from_forms, startup_from_forms_with_inherit, startup_from_source,
    BootstrapArgs, FrozenWorld, ProcessRuntime, StartupError, USER_MAIN_PATH,
};
pub use host::harness::{Harness, HarnessError, Outcome};
pub use hash::{canonical_edn_wat, hash_canonical_ast, hex_encode, verify_source_hash, HashError};
pub use scope::{fresh_scope, Identifier, ScopeId};
pub use lexer::{LexError, LexErrorKind};
pub use load::loader::{
    resolve_loads, FsLoader, InMemoryLoader, LoadError, LoadErrorKind, LoadFetchError, LoadSpec, LoadedSource,
    PayloadInterface, SourceInterface, SourceLoader, VerificationSpec,
};
pub use lower::{lower, LowerError};
pub use macros::{
    expand_all, register_defmacros, MacroDef, MacroError, MacroRegistry,
};
pub use parser::{parse_all_with_file, parse_one_with_file, ParseError, ParseErrorKind};
// The parse_one! and parse_all! macros are exported at crate root via
// #[macro_export] in parser.rs — consumers call them as `wat::parse_one!(src)`.

/// Assert two EDN strings are DATA-equal: parse both via `wat_edn::parse_owned`
/// and compare the parsed `OwnedValue`s. A malformed emission FAILS to parse →
/// the test fails (you cannot green a non-EDN error face). On mismatch the
/// failure message shows the raw strings so the diff is readable.
///
/// Stone C (arc 296): this is the wall — the test proves EDN-ness by PARSING,
/// not by trusting a string. A non-EDN face cannot pass; key-order / whitespace
/// differences in the emission do NOT cause false failures.
///
/// The `$expected` side is typically `include_str!("<test>__<case>.edn")` — a
/// co-located pretty-printed reference file generated from the actual at golden
/// capture time. See `arc 296 stone C` for naming convention.
#[macro_export]
macro_rules! assert_edn_eq {
    ($actual:expr, $expected:expr) => {{
        let a_raw: String = $actual;
        let e_raw: &str = $expected;
        let a_val = ::wat_edn::parse_owned(&a_raw)
            .unwrap_or_else(|err| panic!(
                "STOP-1: ACTUAL is not valid EDN — a non-EDN error face survived stone B.\n\
                 parse error: {}\n\
                 actual: {}", err, a_raw));
        let e_val = ::wat_edn::parse_owned(e_raw)
            .unwrap_or_else(|err| panic!(
                "EXPECTED golden is not valid EDN.\n\
                 parse error: {}\n\
                 expected: {}", err, e_raw));
        assert_eq!(a_val, e_val,
            "EDN data mismatch\n--- actual (raw) ---\n{}\n--- expected (raw) ---\n{}",
            a_raw, e_raw);
    }};
    ($actual:expr, $expected:expr, $msg:expr) => {{
        let a_raw: String = $actual;
        let e_raw: &str = $expected;
        let a_val = ::wat_edn::parse_owned(&a_raw)
            .unwrap_or_else(|err| panic!(
                "STOP-1: ACTUAL is not valid EDN — a non-EDN error face survived stone B.\n\
                 message: {}\n\
                 parse error: {}\n\
                 actual: {}", $msg, err, a_raw));
        let e_val = ::wat_edn::parse_owned(e_raw)
            .unwrap_or_else(|err| panic!(
                "EXPECTED golden is not valid EDN.\n\
                 message: {}\n\
                 parse error: {}\n\
                 expected: {}", $msg, err, e_raw));
        assert_eq!(a_val, e_val,
            "EDN data mismatch ({})\n--- actual (raw) ---\n{}\n--- expected (raw) ---\n{}",
            $msg, a_raw, e_raw);
    }};
}

/// Assert `actual` (an emitted EDN string) is DATA-equal to a co-located `.edn`
/// reference file — the file-backed form of [`assert_edn_eq!`]. The reference
/// lives beside the calling test source (`file!()`-relative, the same resolution
/// [`crate::freeze::startup_beside`] uses for `.wat` fixtures), pretty and
/// legible, generated by CAPTURE — never hand-authored.
///
/// - **normal run:** read the `.edn`, then `assert_edn_eq!(actual, contents)` —
///   parse BOTH, compare `OwnedValue` data. A non-EDN `actual` fails at STOP-1;
///   a lie cannot parse (arc 296 R18 *DATIS NIHIL LATET*). Data-equality is
///   strictly stronger than the old string-eq goldens and than any `contains`
///   check — key-order / whitespace drift never false-fails, but a malformed or
///   wrong-shaped face cannot pass.
/// - **`UPDATE_EDN=1` set:** validate `actual` parses as EDN (STOP-1 if not — a
///   malformed face is NEVER frozen as a golden), then write `actual` VERBATIM to
///   the `.edn`. Capture-don't-guess (arc 298.2's mandated method); the emission
///   is already pretty, so the file is legible and is exactly what was emitted.
///
/// Path: `<dir-of-file!()>/<name>`. Naming convention: `<rs-stem>__<case>.edn`
/// (e.g. `wat_core_cond__cond_refuses_missing_else.edn`). See arc 296 stone C /
/// the recapture swarm.
#[macro_export]
macro_rules! assert_edn_matches_file {
    ($actual:expr, $name:expr $(, $msg:expr)?) => {{
        let a_raw: String = $actual;
        let edn_path = ::std::path::Path::new(file!())
            .parent()
            .expect("file!() should have a parent directory")
            .join($name);
        if ::std::env::var_os("UPDATE_EDN").is_some() {
            // capture-don't-guess (298.2): a non-EDN face is NEVER written as a golden.
            // PRETTY on capture — the assert is DATA-equality (it parses both sides), so the
            // golden's FORMAT is free; pretty-print (wat_edn::write_pretty, 2-space indent) makes
            // it legible instead of a flat one-line blob. The data compared is identical either way.
            let a_val = ::wat_edn::parse_owned(&a_raw).unwrap_or_else(|err| panic!(
                "STOP-1: refusing to capture a non-EDN face as a golden — a non-EDN \
                 error face survived stone B.\n parse error: {}\n actual: {}", err, a_raw));
            let body = format!("{}\n", ::wat_edn::write_pretty(&a_val));
            ::std::fs::write(&edn_path, body)
                .unwrap_or_else(|e| panic!("UPDATE_EDN: failed to write {:?}: {}", edn_path, e));
            eprintln!("UPDATE_EDN: captured {:?}", edn_path);
        } else {
            let expected = ::std::fs::read_to_string(&edn_path).unwrap_or_else(|e| panic!(
                "missing EDN reference {:?}: {}\n\
                 run `UPDATE_EDN=1 cargo nextest run -p wat --test <cluster>` to capture it",
                edn_path, e));
            $crate::assert_edn_eq!(a_raw, expected.as_str() $(, $msg)?);
        }
    }};
}

/// Assert that **some** [`CheckError`](crate::check::error::CheckError) in a
/// `check_program` result set matches a given [`CheckErrorKind`](crate::check::error::CheckErrorKind)
/// pattern (+ optional guard) — **membership**, not `errs[0]` positional indexing.
///
/// `CheckErrors(Vec<CheckError>)` is a *set* of findings; its order is
/// per-process nondeterministic (`HashMap`/`RandomState` reseeds each
/// process run). Matching `errs[0]` is a coin-flip whenever a form emits
/// more than one error. This macro is the canonical replacement: it asserts
/// data equality on set membership (the expected error is *a member of the
/// set*, order-independent) and, on failure, dumps the **whole set** as EDN
/// (`CheckErrors`' `Debug` impl emits EDN — Stone B) so the failure is
/// legible regardless of which slot the match would have landed in.
///
/// Fold the site's exact expected field values into the guard as `==`
/// comparisons — this is the SAME structural assertion the old `errs[0]`
/// match performed, only the positional index is removed. Do NOT loosen to
/// a substring/`contains` check (the loose-assert anti-pattern the arc
/// already cut) — every field named in the guard must be an exact match.
#[macro_export]
macro_rules! assert_check_error_present {
    ($errs:expr, $pat:pat $(if $guard:expr)? $(,)?) => {{
        let __errs = &$errs;
        assert!(
            __errs.iter().any(|__e| matches!(&__e.kind, $pat $(if $guard)?)),
            "no check error matched `{}`; errors were:\n{:?}",
            stringify!($pat $(if $guard)?),
            $crate::check::error::CheckErrors(__errs.iter().cloned().collect::<Vec<_>>()),
        );
    }};
}

pub use resolve::{is_reserved_prefix, resolve_references, ResolveError, UnresolvedReference};
pub use runtime::{
    eval, register_aggregate_methods, register_defines, register_struct_methods,
};
pub use value::{AggregateValue, HolonForm, EncodingCtx, EnvBuilder, Environment, Function, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
pub use types::{
    parse_type_expr, register_stdlib_types, register_types, register_types_with_acronyms,
    AggregateDef, Nature,
    AliasDef, EnumDef, EnumVariant, NewtypeDef, TypeDef, TypeEnv, TypeError, TypeExpr,
};

use holon::{encode, ScalarEncoder, Vector, VectorManager};

/// Unified error type across the parse + lower pipeline.
#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    Lower(LowerError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{}", e),
            Error::Lower(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::Parse(e)
    }
}
impl From<LowerError> for Error {
    fn from(e: LowerError) -> Self {
        Error::Lower(e)
    }
}

/// Evaluate a wat source string containing a single algebra-core form and
/// produce its encoded vector.
///
/// MVP-level convenience for the interpret path: parse → lower → encode.
/// Only algebra-core UpperCalls are supported (no `define`, no `let`, no
/// macros, no user-declared types in this slice). The source is expected
/// to be a single top-level form.
///
/// # Example
///
/// ```
/// use wat::eval_algebra_source;
/// use holon::{ScalarEncoder, VectorManager};
///
/// let vm = VectorManager::with_seed(1_024, 42);
/// let se = ScalarEncoder::with_seed(1_024, 42);
///
/// let src = r#"(:wat::holon::Bind (:wat::holon::Atom "role") (:wat::holon::Atom "filler"))"#;
/// let vector = eval_algebra_source(src, &vm, &se).unwrap();
/// assert_eq!(vector.dimensions(), 1_024);
/// ```
pub fn eval_algebra_source(
    src: &str,
    vm: &VectorManager,
    scalar: &ScalarEncoder,
) -> Result<Vector, Error> {
    let ast = parse_one!(src)?;
    let holon = lower(&ast)?;
    Ok(encode(&holon, vm, scalar))
}

/// Arc 255 STONE-retirement-table-becomes-mechanism — the retirement table's
/// retired-form names, bridged out of the `pub(crate)` `remedy` module for the
/// end-to-end reachability gate (`tests/cli/retirement_table_reachable.rs`), which
/// drives the real `wat` binary and must walk `RETIREMENT_TABLE` itself rather than
/// a hand-maintained copy of its names. Not for production use — mirrors the
/// `#[doc(hidden)] pub fn new_for_test` convention (`src/kernel/peer.rs`).
#[doc(hidden)]
pub fn retirement_table_names_for_gate() -> Vec<&'static str> {
    remedy::retirement_table_names()
}

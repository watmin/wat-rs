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
//!   `lambda`, `struct`, `enum`, `newtype`, `typealias`, `load!`, `set-*!`,
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
//! - [`identifier`] — `Identifier` with `BTreeSet<ScopeId>` scope sets
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
//! - [`runtime`] — AST-walker for `define` / `lambda` / `let` / `if`
//!   + algebra-core dispatch.
//! - [`freeze`] — the 12-step startup pipeline that composes parse →
//!   resolve → check → freeze into a single world.
//! - [`rust_deps`] — `:rust::*` namespace registry + marshaling
//!   traits (`FromWat` / `ToWat`) + `ThreadOwnedCell<T>` /
//!   `OwnedMoveCell<T>` scope primitives.
//! - [`stdlib`] — baked-in wat source files (Subtract, Console,
//!   LocalCache, Cache, …) registered before user code parses.

extern crate self as wat;

pub mod assertion;
pub mod ast;
pub mod check;
pub mod compose;
pub mod config;
pub mod dim_router;
pub mod fork;
pub mod harness;
pub mod freeze;
pub mod hash;
pub mod hologram;
pub mod identifier;
pub mod io;
pub mod lexer;
pub mod load;
pub mod lower;
pub mod macros;
pub mod parser;
pub mod resolve;
pub mod runtime;
pub mod rust_deps;
pub mod sandbox;
pub mod panic_hook;
pub mod source;
pub mod span;
pub(crate) mod stdlib;
pub mod string_ops;
pub mod test_runner;
pub mod time;
pub mod types;
pub mod vm_registry;

pub use compose::{compose_and_run, compose_and_run_with_loader};
pub use source::WatSource;
pub use span::Span;
pub use wat_macros::{main, test};

pub use ast::WatAST;
pub use check::{check_program, CheckEnv, CheckError, CheckErrors, TypeScheme};
pub use config::{
    collect_entry_file, collect_entry_file_with_inherit, CapacityMode, Config, ConfigError,
};
pub use dim_router::{DimRouter, SizingRouter, DEFAULT_TIERS};
pub use vm_registry::{Encoders, EncoderRegistry};
pub use freeze::{
    eval_digest_in_frozen, eval_in_frozen, eval_signed_in_frozen, invoke_user_main,
    startup_from_forms, startup_from_forms_with_inherit, startup_from_source, FrozenWorld,
    StartupError, USER_MAIN_PATH,
};
pub use harness::{Harness, HarnessError, Outcome};
pub use hash::{canonical_edn_wat, hash_canonical_ast, hex_encode, verify_source_hash, HashError};
pub use identifier::{fresh_scope, Identifier, ScopeId};
pub use lexer::LexError;
pub use load::{
    resolve_loads, FsLoader, InMemoryLoader, LoadError, LoadFetchError, LoadSpec, LoadedSource,
    PayloadInterface, SourceInterface, SourceLoader, VerificationSpec,
};
pub use lower::{lower, LowerError};
pub use macros::{
    expand_all, register_defmacros, MacroDef, MacroError, MacroRegistry,
};
pub use parser::{parse_all, parse_one, ParseError};
pub use resolve::{is_reserved_prefix, resolve_references, ResolveError, UnresolvedReference};
pub use runtime::{
    eval, register_defines, register_struct_methods, EncodingCtx, EnvBuilder, Environment,
    Function, RuntimeError, StructValue, SymbolTable, Value,
};
pub use types::{
    parse_type_expr, register_stdlib_types, register_types, AliasDef, EnumDef, EnumVariant,
    NewtypeDef, StructDef, TypeDef, TypeEnv, TypeError, TypeExpr,
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
    let ast = parse_one(src)?;
    let holon = lower(&ast)?;
    Ok(encode(&holon, vm, scalar))
}

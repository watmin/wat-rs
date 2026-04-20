//! wat — the wat language frontend + runtime.
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
//!   Distinct from `holon::HolonAST` — the WatAST carries `define`,
//!   `lambda`, `struct`, `enum`, `newtype`, `typealias`, `load!`, `set-*!`,
//!   `let`, `if`, `match`, `defmacro`, and all the language-level forms.
//!   Algebra-core calls appear as `UpperCall` nodes that are lowered to
//!   `HolonAST` at evaluation time.
//! - [`lexer`] — tokenizer for the s-expression surface. Handles
//!   keyword-path tokens, the colon-quoting rule, string/numeric/bool
//!   literals, comments.
//! - [`parser`] — tokens → `WatAST`.
//! - [`config`] — entry-file discipline + `set-*!` setter commit.
//! - [`load`] — recursive load-form resolution with `:wat::load::*` and
//!   `:wat::verify::*` interface keywords.
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
//! - [`lower`] — `WatAST` algebra-core subtree → `holon::HolonAST`.
//! - [`runtime`] — AST-walker for `define` / `lambda` / `let` / `if`
//!   + algebra-core dispatch.
//!
//! Not yet built: freeze pass (task #139), `:user/main` +
//! constrained eval (task #140), `wat-vm` CLI binary (task #141).

pub mod ast;
pub mod check;
pub mod config;
pub mod freeze;
pub mod hash;
pub mod identifier;
pub mod lexer;
pub mod load;
pub mod lower;
pub mod macros;
pub mod parser;
pub mod resolve;
pub mod runtime;
pub mod rust_deps;
pub mod stdlib;
pub mod types;

pub use ast::WatAST;
pub use check::{check_program, CheckEnv, CheckError, CheckErrors, TypeScheme};
pub use config::{collect_entry_file, CapacityMode, Config, ConfigError};
pub use freeze::{
    eval_digest_in_frozen, eval_in_frozen, eval_signed_in_frozen, invoke_user_main,
    startup_from_source, FrozenWorld, StartupError, USER_MAIN_PATH,
};
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
    eval, register_defines, EncodingCtx, EnvBuilder, Environment, Function, RuntimeError,
    SymbolTable, Value,
};
pub use types::{
    parse_type_expr, register_types, AliasDef, EnumDef, EnumVariant, NewtypeDef, StructDef,
    TypeDef, TypeEnv, TypeError, TypeExpr,
};

use holon::{encode, AtomTypeRegistry, ScalarEncoder, Vector, VectorManager};

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
/// use holon::{AtomTypeRegistry, ScalarEncoder, VectorManager};
///
/// let vm = VectorManager::with_seed(1_024, 42);
/// let se = ScalarEncoder::with_seed(1_024, 42);
/// let reg = AtomTypeRegistry::with_builtins();
///
/// let src = r#"(:wat::algebra::Bind (:wat::algebra::Atom "role") (:wat::algebra::Atom "filler"))"#;
/// let vector = eval_algebra_source(src, &vm, &se, &reg).unwrap();
/// assert_eq!(vector.dimensions(), 1_024);
/// ```
pub fn eval_algebra_source(
    src: &str,
    vm: &VectorManager,
    scalar: &ScalarEncoder,
    registry: &AtomTypeRegistry,
) -> Result<Vector, Error> {
    let ast = parse_one(src)?;
    let holon = lower(&ast)?;
    Ok(encode(&holon, vm, scalar, registry))
}

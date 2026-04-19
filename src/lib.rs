//! wat-rs — the wat language frontend + runtime.
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
//! - [`lower`] — `WatAST` algebra-core subtree → `holon::HolonAST`.
//!
//! Future modules: `config` (set-*! + config pass), `load` (recursive
//! load! resolution), `macro_expand` (defmacro hygiene), `types` (type
//! env), `resolve` (name resolution), `check` (rank-1 HM), `hash`
//! (canonical-EDN + cryptographic verification), `runtime` (AST walker).

pub mod ast;
pub mod config;
pub mod identifier;
pub mod lexer;
pub mod load;
pub mod lower;
pub mod macros;
pub mod parser;
pub mod runtime;

pub use ast::WatAST;
pub use config::{collect_entry_file, CapacityMode, Config, ConfigError};
pub use identifier::{fresh_scope, Identifier, ScopeId};
pub use lexer::LexError;
pub use load::{
    resolve_loads, FsLoader, InMemoryLoader, LoadError, LoadFetchError, LoadSpec, LoadedSource,
    SourceLoader, VerificationMode,
};
pub use lower::{lower, LowerError};
pub use macros::{
    expand_all, register_defmacros, MacroDef, MacroError, MacroRegistry,
};
pub use parser::{parse_all, parse_one, ParseError};
pub use runtime::{
    eval, register_defines, EnvBuilder, Environment, Function, RuntimeError, SymbolTable, Value,
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
/// ```ignore
/// use wat_rs::eval_algebra_source;
/// use holon::{AtomTypeRegistry, ScalarEncoder, VectorManager};
///
/// let vm = VectorManager::with_seed(10_000, 42);
/// let se = ScalarEncoder::with_seed(10_000, 42);
/// let reg = AtomTypeRegistry::with_builtins();
///
/// let src = r#"(:wat/algebra/Bind (:wat/algebra/Atom "role") (:wat/algebra/Atom "filler"))"#;
/// let vector = eval_algebra_source(src, &vm, &se, &reg).unwrap();
/// assert_eq!(vector.dimensions(), 10_000);
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

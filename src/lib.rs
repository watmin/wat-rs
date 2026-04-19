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
pub mod lexer;
pub mod lower;
pub mod parser;

pub use ast::WatAST;

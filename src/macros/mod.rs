//! `defmacro` — parse-time macro expansion with Racket sets-of-scopes
//! hygiene (Flatt 2016).
//!
//! Per 058-031: macros transform source forms BEFORE hashing, signing,
//! type-checking, or evaluation. Two source files that differ only in
//! macro aliases (e.g. `Subtract` vs `Blend _ _ 1 -1`) expand to the
//! same canonical AST and the same hash — the substrate commit of
//! hash-IS-identity holds.
//!
//! # File map
//!
//! - `registry` — storage (`MacroDef`, `MacroRegistry`)
//! - `parse`    — form → `MacroDef` + registration helpers
//! - `expand`   — call-site → AST (template walk, hygiene, fixpoint)
//! - `eval`     — the fenced expand-time evaluator (`macro_eval`; default-deny pure-total gate)
//! - `error`    — `MacroError` / `MacroErrorKind`
//!
//! # Hygiene by construction
//!
//! A macro that introduces a name (`(let ((tmp ,x)) ...)`) cannot
//! collide with a user's `tmp` in the caller's scope. The mechanism,
//! per FOUNDATION's specified algorithm:
//!
//! 1. At each macro invocation, allocate a fresh [`ScopeId`].
//! 2. Walk the macro's template. Every identifier whose origin is the
//!    template source has the fresh scope added to its scope set.
//! 3. Identifiers that came in via macro arguments (substituted at
//!    `,x` unquote sites) KEEP their original scope sets.
//! 4. Lexical-scope lookup compares `(name, scope_set)` pairs — so
//!    `tmp[{macro-scope}]` and `tmp[{user-scope}]` resolve to distinct
//!    bindings.
//!
//! Variable capture is structurally impossible (not discipline-enforced)
//! for quasiquote-template bodies via the scope-set mechanism above.
//! Program-body macros (arc 249.2b-ii) enforce hygiene by a default-deny
//! refusal: `check_program_body_hygiene` (expand.rs, gate E) rejects any
//! program body whose quasiquote introduces a literal binder name.
//!
//! # What this slice supports
//!
//! - `defmacro` forms with quasiquote-template bodies: `` ` `` for the
//!   template, `,expr` for parameter substitution, `,@expr` for list
//!   splicing.
//! - Fixpoint expansion (macros expand to more macros until no more
//!   remain). Depth limit prevents pathological infinite expansion.
//! - Full hygiene for the classic capture pattern.
//! - Threading macros `->` (thread-first) and `->>` (thread-last): ordinary
//!   registered wat macros in `wat/core.wat`; rehomed from Rust desugars in
//!   arc 249.3.
//! - `keyword/of` special form: constructs parametric keywords
//!   (e.g. `(:wat::core::keyword/of :Head :Arg)` → `:Head<Arg>`);
//!   ordinary registered wat macro in `wat/core.wat`; rehomed from Rust
//!   desugar in arc 249.4.
//! - Computed-unquote `,(expr)`: a List whose head is a Keyword is
//!   evaluated at expand-time via `macro_eval` (the default-deny
//!   pure-total fenced evaluator; arc 249.2b-i, building on the
//!   arc 143 slice 2 computed-unquote path) with macro params substituted.
//!
//! # Scope
//!
//! This slice handles quasiquote-template + program-body macro expansion.
//! Typed-macro `:AST<T>` checking lives in the type checker's phase
//! (058-032).

use crate::ast::WatAST;

pub(crate) mod error;
pub(crate) mod registry;
pub(crate) mod parse;
pub(crate) mod expand;
pub(crate) mod eval;

pub use error::{MacroError, MacroErrorKind};
pub use registry::{MacroDef, MacroRegistry};
pub use expand::{expand_all, expand_once};
pub use parse::{register_defmacros, register_stdlib_defmacros};
pub use expand::EXPANSION_DEPTH_LIMIT;

/// A batch of expanded forms, or the macro error that stopped expansion.
pub(super) type ExpandBatch = Result<Vec<WatAST>, MacroError>;

#[cfg(test)]
mod tests;

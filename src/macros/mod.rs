//! `defmacro` — parse-time macro expansion with Racket sets-of-scopes
//! hygiene (Flatt 2016).
//!
//! Per 058-031: macros transform source forms BEFORE hashing, signing,
//! type-checking, or evaluation. Two source files that differ only in
//! macro aliases (e.g. `Subtract` vs `Blend _ _ 1 -1`) expand to the
//! same canonical AST and the same hash — the substrate commit of
//! hash-IS-identity holds.
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
//! Variable capture is structurally impossible, not discipline-enforced.
//!
//! # What this slice supports
//!
//! - `defmacro` forms with quasiquote-template bodies: `` ` `` for the
//!   template, `,expr` for parameter substitution, `,@expr` for list
//!   splicing.
//! - Fixpoint expansion (macros expand to more macros until no more
//!   remain). Depth limit prevents pathological infinite expansion.
//! - Full hygiene for the classic capture pattern.
//!
//! # What's deferred
//!
//! - Arbitrary-Lisp macro bodies (computed conditional templates,
//!   macro-authoring helpers beyond quasiquote). The spec admits them
//!   but the common case — and every 058 stdlib macro — uses
//!   quasiquote alone.
//! - Typed-macro checking (058-032). Macro parameters here are
//!   positional AST arguments; the type checker validates `:AST<T>`
//!   annotations against body positions in its own phase.

pub mod error;
pub(crate) mod registry;
pub(crate) mod parse;
pub(crate) mod expand;

pub use error::{MacroError, MacroErrorKind};
pub use registry::{MacroDef, MacroRegistry};
pub use parse::{expand_once, register_defmacros, register_stdlib_defmacros};
pub use expand::expand_all;
pub use parse::EXPANSION_DEPTH_LIMIT;

#[cfg(test)]
mod tests;

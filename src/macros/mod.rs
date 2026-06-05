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
//! - Threading macros `->` (thread-first) and `->>` (thread-last):
//!   transitional in-pass Rust desugars, rehomed to wat code in arc
//!   249.3/249.4.
//! - `keyword/of` special form: constructs parametric keywords
//!   (e.g. `(:wat::core::keyword/of :Head :Arg)` → `:Head<Arg>`);
//!   transitional in-pass Rust desugar, rehomed in arc 249.4.
//! - Bounded `for`-comprehension in splice position:
//!   `,@(:wat::core::for [x xs] tmpl)` — iterates a finite list and
//!   instantiates the template per element (arc 248 slice 1).
//! - Computed-unquote `,(expr)`: a List whose head is a Keyword is
//!   evaluated at expand-time via `runtime::eval` with macro params
//!   substituted (arc 143 slice 2).
//!
//! # What's deferred
//!
//! - Arbitrary conditional macro bodies beyond quasiquote (conditionals,
//!   recursive macro helpers). The spec admits them but the common case —
//!   and every 058 stdlib macro — uses quasiquote alone.
//! - Typed-macro checking (058-032). Macro parameters here are
//!   positional AST arguments; the type checker validates `:AST<T>`
//!   annotations against body positions in its own phase.

pub(crate) mod error;
pub(crate) mod registry;
pub(crate) mod parse;
pub(crate) mod expand;
pub(crate) mod eval;

pub use error::{MacroError, MacroErrorKind};
pub use registry::{MacroDef, MacroRegistry};
pub use expand::{expand_all, expand_once};
pub use parse::{register_defmacros, register_stdlib_defmacros};
pub use parse::EXPANSION_DEPTH_LIMIT;

#[cfg(test)]
mod tests;

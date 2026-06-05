//! # Scope — warded home for wat's lexical-scope identity machinery.
//!
//! This home holds wat's lexical-scope machinery — the `Identifier`/`ScopeId`
//! primitives that make macro expansion *hygienic* per Racket's sets-of-scopes
//! model (Flatt 2016). The scope-aware resolution policy lands here in stone
//! 249.5b.
//!
//! ## Why this module exists
//!
//! Stone 249.5a — lifts `src/identifier.rs` into this warded home. The
//! primitives that discriminate lexical identity (`Identifier` = name +
//! `BTreeSet<ScopeId>`, `ScopeId`, `fresh_scope`) belong under a single roof
//! so that scope-resolution machinery has a durable neighbor rather than
//! growing back into the flat `src/`.
//!
//! ## Contents
//!
//! - `identifier.rs` — `Identifier`, `ScopeId`, `fresh_scope`: the bare
//!   sets-of-scopes primitives. The parser emits `Identifier::bare` (empty
//!   scope set); the macro expander mints fresh `ScopeId`s and calls
//!   `add_scope` on template-originated identifiers.
//! - `resolution.rs` — `env_key`: scope-aware environment key derivation.
//!   Stone 249.5b — makes the expander's scope tags load-bearing at runtime
//!   lookup, preventing classic macro variable capture.

pub mod identifier;
pub mod resolution;

pub use identifier::{fresh_scope, Identifier, ScopeId};
pub use resolution::{env_key, scoped_arg_names};

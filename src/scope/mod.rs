//! vigilatum: 2026-06-05T21:37:24Z — UPDATED-vigilia 10-spell guard L1+L2=0 (universal-7:
//! intueri/solvere/conformare/purgare/struere/sequi/temperare + exigere +
//! excusare + circumspicere-last; conditional triggers weighed: perspicere/mora
//! not fired, secare not mustered — the lone AtomicU64 is a monotonic mint, no
//! parallel slot-writes), clippy-clean in-home. Two full inward rounds + the
//! perimeter; every finding fought (the one stale-guard rune was FIXED out of
//! existence, not reworded). Declared invariants, each enforced by a living
//! gate: (1) only `env_key` + the canonical hasher read the Identifier scope-set
//! (tests/probe_hygiene_scopes_reader_gate.rs); (2) walk_template scope-set
//! uniformity — binder ≡ body-reference (macros/tests.rs); (3) 2-scope
//! end-to-end resolution under nested expansion (tests/probe_macro_hygiene_capture.rs);
//! (4) raw control characters rejected at lex — the U+0001 separator invariant
//! is enforced, not conventional (src/lexer.rs).
//!
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
pub use resolution::env_key;

//! Rete engine — Rust-primitive home for the rete network's pure operations.
//!
//! ## Why this module exists
//!
//! Arc 278 — the rules engine. The rete home mirrors `src/collection/` in layout:
//! a `mod.rs` warding the home boundary + a `matcher.rs` carrying the single-fact
//! alpha-match primitive. New rete Rust primitives land here; the WAT-level engine
//! (the session, beta network, join layer) rides on top.
//!
//! ## Stone map
//! - **Stone 2a** (`matcher.rs`) — `eval_alpha_match`: given a condition form (DATA)
//!   and a fact (record), return `Some(bindings)` iff the fact's type matches the
//!   condition head AND every clause holds. Pure: no `Environment`, no `eval_inner`.
//! - Stone 2b — alpha-memory (`insert`); consumes `eval_alpha_match`.
//! - Stone 3 — cross-fact join (beta network); builds on alpha-memory.
//! - **Stone 4a** (`matcher.rs`) — `eval_insert`: given an insert form (DATA, a quoted
//!   `(:wat::rete::insert (:RecordType arg…))`) and a token's bindings map, resolve each
//!   fact-arg via `resolve_operand` (?var + literal only; no current fact) and return the
//!   derived `:wat::Record`. The RHS dual of `eval_alpha_match`. Raises on malformed form /
//!   unresolved operand (never silently drops).
//!
//! ## Declaration sites
//!
//! - **Runtime dispatch:** `":wat::rete::alpha-match"` arm and `":wat::rete::eval-insert"` arm
//!   in `dispatch_keyword_head_value` (`src/runtime.rs`) route here.
//! - **Check scheme:** registered in `register_builtins` (`src/check.rs`) —
//!   `alpha-match`: `[:wat::WatAST, :wat::Record] -> Option<PersistentMap<String, Value>>`.
//!   `eval-insert`: `[:wat::WatAST, :wat::core::PersistentMap] -> :wat::Record`.

pub(crate) mod matcher;

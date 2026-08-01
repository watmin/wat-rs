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
//!   derived `:wat::core::Record`. The RHS dual of `eval_alpha_match`. Raises on malformed form /
//!   unresolved operand (never silently drops).
//!
//! ## Declaration sites
//!
//! - **Runtime dispatch:** `":wat::rete::alpha-match"` arm and `":wat::rete::eval-insert"` arm
//!   in `dispatch_keyword_head_value` (`src/runtime.rs`) route here.
//! - **Check scheme:** registered in `register_builtins` (`src/check.rs`) —
//!   `alpha-match`: `[:wat::WatAST, :wat::core::Record] -> Option<PersistentMap<String, Value>>`.
//!   `eval-insert`: `[:wat::WatAST, :wat::core::PersistentMap] -> :wat::core::Record`.

pub(crate) mod matcher;
// Arc 294 item 9a (DESIGN-rete-defrule-wall.md) — the freeze-time `defrule` wall: validates
// every rule's quoted :when/:then against the type registry (post-register) and reorders
// :then kwargs to declaration order, so the 9a-corruption class (unrecognized clause /
// unknown field-ref / scrambled kwargs RHS) becomes a LOCATED freeze error instead of a
// silent runtime `None` or scrambled fact. Shares its clause grammar with `matcher.rs`'s
// `classify_rete_clause` — one grammar, two consumers (design call 1).
pub(crate) mod validate;
// Stone 5b (collect.rs) — eval_collect_rules: reflect the symbol table for a namespace's defrule'd
// zero-arg rule fns (ret_type :wat::rete::Rule), invoke each → PersistentVector<Rule>.
pub(crate) mod collect;
// Stone P1 (kernel.rs) — WorkingMemory: native mutable mirror of Session + to_transient/to_persistent
// lossless boundary. Sealed Rust; no wat surface. Fire kernel (P2–P5) mutates this; user calls fire.
pub(crate) mod kernel;
// Stone 6a (purity.rs) — default-deny purity classifier: is_pure_expr / is_pure_fn (transitive,
// cycle-safe) + eval_pure_predicate (the :wat::rete::pure? primitive entry point).
pub(crate) mod purity;
// DESIGN-STONE-alpha-discrimination-tree.md — AlphaTree: replaces the P8 linear
// "every alpha of this fact's type" scan with a root-to-leaf walk over provable equality
// discriminators. Prune-only (candidate set; `alpha_match_inner` stays the sole authority),
// alpha-only (beta stays runtime), built once at setup from the immutable network.
pub(crate) mod alpha_tree;

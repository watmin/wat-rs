//! Rete engine — Rust-primitive home for the rete network's pure operations.
//!
//! ## Why this module exists
//!
//! Arc 278 — the rules engine. The rete home mirrors `src/collection/` in layout:
//! a `mod.rs` warding the home boundary + a `matcher.rs` carrying the single-fact
//! alpha-match primitive (oracle / differential). Native fire is `kernel/`.
//! The wat files are compile + `$oracle` reference, not the production fire path.
//!
//! ## Stone map
//! - **Stone 2a** (`matcher.rs`) — `alpha_match_inner` is the oracle / differential matcher's
//!   pure core (no `Environment`, no `eval_inner`). Native fire uses compiled exec
//!   (`exec_compiled_with_key_ids`). The wat entry points (`alpha-match`/`alpha-match-local`/
//!   `alpha-match-under`) are `#[wat_intrinsic]`-homed in `src/intrinsic/rete.rs` (arc 255
//!   Stone P6-c-W5a) — `matcher.rs` no longer carries a hand-rolled dispatch fn for them.
//! - Stone 2b — alpha-memory (`insert`); consumes `alpha_match_inner`.
//! - Stone 3 — cross-fact join (beta network); builds on alpha-memory.
//! - **Stone 4a** (`eval_insert.rs`) — `eval_insert`: given a fact form (DATA, a quoted
//!   `(:RecordType arg…)` — arc 278 Stone A dropped the `insert` RHS-marker wrapper) and a
//!   token's bindings map, resolve each fact-arg via `resolve_rhs_value` (`?var`, literal, or
//!   fenced List; fn-headed items are `CompiledRhs::Call` / `build_insert_fact_call`) and return the
//!   derived `:wat::core::Record`. The RHS dual of `alpha_match_inner`. Raises on malformed form /
//!   unresolved operand (never silently drops).
//!
//! ## Declaration sites
//!
//! - **Runtime dispatch:** `":wat::rete::alpha-match"` (and `alpha-match-local`/
//!   `alpha-match-under`) are registered via `#[wat_intrinsic]` in `src/intrinsic/rete.rs` —
//!   the registry-first door in `dispatch_keyword_head`/`dispatch_keyword_head_value`
//!   (`src/runtime.rs`) finds them before the giant match is ever reached. `":wat::rete::
//!   eval-insert"` is still a hand-rolled arm there and routes here.
//! - **Check scheme:** registered in `register_builtins` (`src/check.rs`) —
//!   `alpha-match`: `[:wat::WatAST, :wat::core::Record] -> Option<PersistentMap<String, Value>>`.
//!   `eval-insert`: `[:wat::WatAST, :wat::core::PersistentMap] -> :wat::core::Record`.

pub(crate) mod matcher;
// Rete-DSL clause grammar — compile/validate/stratify consume this, not the oracle matcher.
pub(crate) mod clause;
// `:then` fact construction (interpreter / differential). Native fire uses compiled_rhs.
pub(crate) mod eval_insert;
// Fenced `where` / RHS expr eval under token bindings.
pub(crate) mod eval_test;
// Explain DerivationStep payload (`step-payload`).
pub(crate) mod step_payload;
// Arc 294 item 9a (DESIGN-rete-defrule-wall.md) — the freeze-time `defrule` wall: validates
// every rule's quoted :when/:then against the type registry (post-register) and reorders
// :then kwargs to declaration order, so the 9a-corruption class (unrecognized clause /
// unknown field-ref / scrambled kwargs RHS) becomes a LOCATED freeze error instead of a
// silent runtime `None` or scrambled fact. Shares its clause grammar with `clause.rs`'s
// `classify_rete_clause` — one grammar, two consumers (design call 1).
pub(crate) mod validate;
// Stone 5b (collect.rs) — eval_collect_rules: reflect the symbol table for a namespace's defrule'd
// zero-arg rule fns (ret_type :wat::rete::Rule), invoke each → PersistentVector<Rule>.
pub(crate) mod collect;
// Stone P1 (`kernel/`) — FireSession + fire + arm intern + insert.
// Sealed Rust. Fire kernel (P2–P5) mutates `FireSession`.
pub(crate) mod kernel;
// Stone 6a (purity.rs) — default-deny purity classifier: is_pure_expr / is_pure_fn (transitive,
// cycle-safe). The `:wat::rete::pure?`/`deterministic?`/`total?`/`primitive?` wat entry points
// are `#[wat_intrinsic]`-homed in `src/intrinsic/rete.rs` (arc 255 Stone P6-c-W5a), not a
// hand-rolled dispatch fn here any more.
pub(crate) mod purity;
// DESIGN-STONE-alpha-discrimination-tree.md — AlphaTree: prune-only candidate set.
// Native authority is compiled exec; `alpha_match_inner` is the oracle / differential.
pub(crate) mod alpha_tree;
// (b) WhereDiscNode — armed `where` circuits. Token → candidate TestNodes.
// Over-approx only. `exec_where` stays the authority.
pub(crate) mod where_tree;
// DESIGN-STONE-compiled-conditions.md — compiles each alpha condition ONCE at setup (beside the
// tree) into a pre-resolved instruction sequence (field indices, slot indices, no per-fact
// string-key allocation, no per-fact classify_rete_clause re-derivation). Not a perf stone (see
// the module doc's amendment note): `alpha_match_inner` remains the reference implementation and
// the differential's other half; this is the mechanism that stops it being re-derived dynamically.
pub(crate) mod compiled_cond;
// DESIGN-STONE-compiled-rhs.md — compiles each rule's :then insert-form(s) ONCE at setup (beside
// compiled_conds) into `CompiledRhs::Record { class, names, ops }` or `CompiledRhs::Call(Program)`
// (no per-fact form re-validation, no per-fact kwargs re-detection, no per-fact ?var key
// re-allocation). `compile_rhs` returning `None` is a setup refuse (`rhs_must_compile`);
// `build_insert_fact` remains the interpreter / differential. This is the mechanism that stops
// the RHS's static program being re-derived dynamically.
pub(crate) mod compiled_rhs;
// DESIGN-STONE-the-one-expression-core — one Expr DAG, three adjacent flips.
// This module is the core. `where` is the first consumer. No Interp arm.
pub(crate) mod expr_ir;
// Arc 278 #55 (S3b+S4) slice one — THE ONE table of rete-namespaced vocabulary ops
// (`RETE_OPS`), the module-set admission test (`RETE_MODULES` / `rete_vocabulary_admitted`),
// and the generic per-class shapes `check.rs`/`runtime.rs`/`purity.rs` iterate. See the
// module doc for the full contract (one list, three readers — STOP-2).
pub(crate) mod vocabulary;
// Arc 278 § 4.1 — the (row x call-site kind) reachability ledger over `RETE_OPS`. Test-only:
// it drives synthesized rules through the real load path to answer "can a user actually get
// here", which the purity/totality/arity/type gates never ask. See the module doc for why the
// unit is the CELL and not the row.
#[cfg(test)]
mod reachability {
    include!("reachability.rs");
}
// `#wat.rete/Export` — compiled program as one EDN value. Native fire only.
pub(crate) mod export;

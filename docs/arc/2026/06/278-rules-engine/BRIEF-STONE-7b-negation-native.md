# BRIEF — Stone 7-b: negation (`:not`/NegationNode) in the NATIVE kernel + the differential

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

The oracle already negates (7-a, `wat/rete.wat`); compile (shared) already mints NegationNodes. The NATIVE
delta engine (`src/rete/kernel.rs` `fire_fixpoint_delta`) has a TestNode filter (6b-ii-b, step 3.5) but no
NegationNode filter → it under-derives → the differential is RED. **Generalize the native filter** to also
handle NegationNode: for each NegationNode, for each NEW token at `d_beta[parent]`, pass it (to
`wm.beta[neg]` + `d_beta[neg]`) iff there is **NO** `token_element_compatible` element in the negated
condition's FULL alpha-memory `wm.alpha[neg_alpha_id]`. Reuse the native `token_element_compatible`
(`kernel.rs:722`). The differential (`fire-rules` native == `fire-rules-spec` oracle on a `:not` rule) is
the gate. Contract: `DESIGN-STONE-7-negation.md` (7-b entry).

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-7-negation.md` — the 7-b entry + the scope (one-sided
   via replay; check the FULL negated alpha-memory, not a delta).
2. **`wat/rete.wat` `filter-pass`** (7-a, just shipped, ~`:1100`) — the ORACLE model: the `NegationNode`
   branch checks `alpha-memory[neg-alpha-id]` for any `token-element-compatible?`, passes iff none. The
   native must match it.
3. **`src/rete/kernel.rs` `fire_fixpoint_delta`** step 3.5 — the 6b-ii-b TestNode filter. **Generalize it**
   to dispatch by node kind: `TestNode` → the existing `eval_test_core` filter; `NegationNode` → the
   negation filter. (Or add a sibling NegationNode loop right after — but a single kind-dispatching pass
   mirrors the oracle's unified `filter-pass` and is preferred.)
4. `src/rete/kernel.rs` `node_kind_label` (`:409`) — add the `"NegationNode"` arm; `node_children`
   (`:442`) — add the NegationNode arm (children at `sf[2]`; NegationNode is `id(0) negated-alpha-id(1)
   children(2)`).
5. `src/rete/kernel.rs` `token_element_compatible` (`:722`) — REUSE for the native compatibility check.
   Read its signature (Token, Element/Value) and how `wm.alpha` stores elements.
6. The negated alpha id: a NegationNode's `struct_form` is `id(0), negated_alpha_id(1), children(2)`;
   extract `negated_alpha_id` from `sf[1]` (a `Value::i64`). `wm.alpha[negated_alpha_id]` is the FULL
   cumulative alpha-memory (all matching facts this fire) — check it, NOT a delta.
7. `tests/probe_arc278_7b_negation_native_differential.rs` — the 4 assertions to green (do NOT edit it).

## The one difference from the test filter (read carefully)

The TestNode filter checks a per-token predicate (just the token). The NegationNode filter checks the token
against the **FULL negated alpha-memory** (`wm.alpha[neg_alpha_id]`, cumulative — "is there ANY matching
fact?"). The new tokens to filter still come from `d_beta[parent]` (the delta), but the absence check is
against the full `wm.alpha`. (Within a fire, the alpha pass populates `wm.alpha` before the filter pass, so
it is complete for base-fact negation — the v1 scope; stratified derived-fact negation is banked.)

## Blast radius (bounded)

- `src/rete/kernel.rs` ONLY (`node_kind_label` + `node_children` arms + the negation branch in the
  filter pass). `token_element_compatible` + `eval_test_core` already exist. NO `wat/rete.wat` (oracle is
  the frozen reference), NO `matcher.rs`/`runtime.rs`/`check.rs`. No new `Value` variant.

## STOP triggers (halt + surface; do not improvise)

1. If `token_element_compatible`'s native signature/contract can't be reused for the negation check —
   STOP, report it.
2. If generalizing the step-3.5 filter to dispatch TestNode vs NegationNode needs restructuring beyond a
   kind branch — STOP, describe it.
3. If the differential stays RED (native ≠ oracle) and you can't localize it — STOP, report native vs
   oracle counts + hypothesis; do NOT weaken the probe.
4. If greening needs editing `wat/rete.wat` — STOP (the oracle is the frozen reference).

## Done = green

`cargo test --release -p wat --test probe_arc278_7b_negation_native_differential` → 4/4 (native==oracle:
1 absent, 0 present-matching, 1 present-different). AND `--test probe_arc278_6b_ii_b_where_native_differential`
→ 4/4 (the native test filter not regressed). AND `--test probe_arc278_northstar_cold_and_windy --
--include-ignored` → 1/0. Then `cargo build --release` clean + `cargo test --release -p wat --lib --
--test-threads=1` → 941/36.

# BRIEF — Stone 7-a: negation (`:not` / NegationNode) in the ORACLE

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

Teach the wat oracle (`wat/rete.wat`) `(:wat::rete::not (:FactType <clause>…))` — a NegationNode. It is a
**hash-join, inverted**: left = the parent token stream, right = the negated condition's alpha-memory; a
token passes downstream iff there are **ZERO** `token-element-compatible?` elements in that alpha-memory
(no matching fact for the token's shared bindings). Reuse the existing join machinery
(`token-element-compatible?`); replay makes it one-sided (no two-sided delta). This is the ORACLE only —
the native port + differential are 7-b. Contract: `DESIGN-STONE-7-negation.md`.

## Read in order (the rooms — all in `wat/rete.wat`)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-7-negation.md` — the scope (Option A: one-sided,
   one node, base-fact correct; stratification/`:exists`/leading-`:not` banked).
2. The node records + `Node` defenum (~`:69-120`) + the `TestNode` added in 6b-ii-a — add a `NegationNode`
   beside it: `(:wat::Record::def :wat::rete::NegationNode [id <- :wat::core::i64  negated-alpha-id <- :wat::core::i64  children <- :wat::core::PersistentVector<wat::core::i64>])`
   and a `:NegationNode [node <- :wat::rete::NegationNode]` variant; add the `node-children` arm.
3. `compile-condition` (the `where`-branch added in 6b-ii-a is the MODEL) — add a second top-branch:
   detect `(:wat::rete::not <inner>)` (head-name `":wat::rete::not"`, via the same `ast->children` +
   `ast-name` you used for `where`). On `:not`: extract `<inner>` (the 2nd child), `find-or-mint-alpha`
   for `<inner>` (so the alpha pass populates its matching facts) → `neg-alpha-id`, mint a `NegationNode`
   carrying `neg-alpha-id`, wire `parent → negation` (parent must be ≥ 0 — a leading `:not` is banked;
   if `parent < 0`, raise a compile error "negation must follow a binding condition"), advance parent =
   negation-id. (No fence — the negated pattern is data, not an expr.)
4. `token-element-compatible?` (`:805-855`) + `cross-join-node` (`:857`) + `hash-join-pass` (`:885`) — the
   join model. The negation check is `token-element-compatible?` over `alpha-memory[neg-alpha-id]`,
   **inverted**: pass the (un-extended) token iff NO element is compatible.
5. The 6b-ii-a **test-pass** in `fire-once` (~`:1020`) — **generalize it into a `filter-pass`**: one fold
   over node-ids (topological order) that dispatches by node kind — `TestNode` → the existing eval-test
   filter; `NegationNode` → the negation filter (append the token to `beta[neg-id]` iff no compatible
   element in `alpha-memory[neg-alpha-id]`). Replace the standalone test-pass fold in `fire-once` with this
   unified filter-pass (so any interleaving of `where`/`:not` in a chain is correct — each filter reads its
   parent's beta, populated earlier in the same topological fold). The negation filter needs alpha-memory →
   pass the alpha-memory (`new-amem`) into the filter-pass alongside beta.
6. `tests/probe_arc278_7a_negation_oracle.rs` — the 3 assertions to green (do NOT edit it).

## Blast radius (bounded)

- `wat/rete.wat` ONLY. NO Rust (`token-element-compatible?` already exists; native is 7-b). Do NOT touch
  the `render-dag` compound-concat fixture.

## STOP triggers (halt + surface; do not improvise)

1. If `find-or-mint-alpha` can't mint an alpha for the inner negated condition `<inner>` (shape differs
   from a normal condition) — STOP, report what it expects.
2. If generalizing the test-pass into a kind-dispatching filter-pass needs restructuring beyond a fold
   over node-ids — STOP, describe it.
3. If `token-element-compatible?` can't be reused for the negation compatibility check — STOP, report its
   actual contract.
4. If greening needs Rust (`kernel.rs`/`matcher.rs`) — STOP (that's 7-b).

## Done = green

`cargo test --release -p wat --test probe_arc278_7a_negation_oracle` → 3/3. AND `--test
probe_arc278_6b_ii_a_where_oracle` → 5/5 (the filter-pass unification must not regress `where`). AND
`--test probe_arc278_northstar_cold_and_windy -- --include-ignored` → 1/0. Then the floors (EXPECTATIONS).

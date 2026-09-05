# DESIGN — Stone 7-exists: `:exists` (existential) = NegationNode's filter predicate, flipped

> **2026-08-17.** This stone's gather is the contract we returned to.
> Implementation drifted: both mouths scanned the **session fact bag**
> (leftover `?v < ?m` after accum became the universal algorithm).
> Dirty tree (`wat/rete.wat` + `src/rete/kernel/`) probes **alpha**
> for fact-shaped inners again. Live breadcrumb:
> **`CURRENT-STATE-annihilate-interpretation.md`**.
>
> Two lines below are **stale**: (1) "Leading `:exists` raises at
> compile" — Clara `test-simple-exists` made leading exists legal.
> (2) "ONE strike, already shipped" — the gather drifted after this
> stone landed; cut 1 is the return, uncommitted.

## What + why
`(:wat::rete::exists <inner>)` is a LHS condition that passes its parent token **iff ≥1 element matches the
inner condition** for the token's bindings — **binds nothing, fires the token exactly once** regardless of how
many match (no multiplicity — the defining difference from a join). It completes the boolean set
(`:and` implicit, `:not` shipped, `:exists` here). The DESIGN keep/cut inventory had it as "KEEP as sugar";
this builds it.

It is **NegationNode, with the filter predicate inverted**: negation passes iff ZERO compatible elements;
exists passes iff ≥1. Same gather (`token-element-compatible?` over the inner alpha-memory), opposite verdict.

## The one contract decision: a sibling `ExistsNode`, not a flag on NegationNode
Add a NEW `ExistsNode` record (same shape as `NegationNode`: `id`, `alpha-id`, `children`) + a `Node` variant
+ a `node-children` arm + a filter-pass arm. **Additive** — does NOT touch `NegationNode`'s arity (no ripple
to its constructors / `sf` indices). The filter-pass already dispatches by `kind`; `ExistsNode` is one more
kind whose predicate is the inverse. (The "flip" is the predicate, expressed as a sibling node — the
cleanest, least-ripple form of "NegationNode flip".)

## Semantics (pin)
- Head `:wat::rete::exists` — collision-free: starts `":"`, disjoint from `"?"` (accumulate) and distinct from
  `:where`/`:not` by exact string match.
- Pass the parent token **unchanged** (no new bindings) iff ≥1 element in `alpha-memory[inner-alpha-id]` is
  `token-element-compatible?` with the token. Otherwise drop it.
- **No multiplicity:** the token passes at most ONCE — it is not extended/multiplied per matching element
  (that is what makes it existential, not a join).
- Leading `:exists` (parent < 0) is unsupported → raise at compile (same guard as `:not`).
- Pure replay dissolves any delta concern (the inner alpha-memory is fixed within a fire), exactly as `:not`.

## Scope — ONE strike (oracle + native + differential)
This is a mechanical flip of an already dual-built stone (7), so oracle + native + the differential ship
together; the differential (`native == oracle`) keeps it honest within the strike. Out of scope (rejected,
not deferred): a flag-on-NegationNode encoding (a sibling node is cleaner); `:exists` binding variables (it
binds nothing, by definition).

## Files
- `wat/rete.wat` — `ExistsNode` record + `Node` variant + `node-children` arm; `compile-condition` `is-exists`
  branch (mirror the `:not` branch); filter-pass `ExistsNode` arm (mirror the NegationNode arm, predicate
  flipped to `any-compat`).
- `src/rete/kernel/` — `node_children` `ExistsNode` arm (`kernel/node.rs`); `fire_fixpoint_delta` filter-pass `ExistsNode`
  branch (pass iff ≥1 `token_element_compatible`).
- `tests/probe_arc278_7exists_native_differential.rs` — the contract (RED at HEAD; 5 tests).

## Done = the differential 5/5 + no regressions (see EXPECTATIONS).

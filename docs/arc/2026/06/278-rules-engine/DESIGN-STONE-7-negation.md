# DESIGN — Stone 7: negation (`:not`) — the NegationNode

> **2026-08-17.** The gather this stone named (`token-element-compatible?`
> over the negated alpha) is what the dirty tree does again for
> fact-shaped inners. Session-fact scan was the drift; see
> **`CURRENT-STATE-annihilate-interpretation.md`**. Combinator /
> `:where` inners still use `exists-cond-under`. **Status line
> below is historical.**

**Status:** STRIKE-READY (7-a drawn; 7-b follows)
**Depends on:** the join machinery (3a/3b — `token-element-compatible?`, alpha-memory) + the filter-pass
shape (6b-ii TestNode). Same dual-impl pattern.

## Why

A rule's LHS needs *absence*: "fire iff there is NO matching fact." `(:wat::rete::not (:Maintenance (?loc
<- :location)))` — fire the alert iff no Maintenance record exists at this location. This is Clara's
negation; with `:exists` = `(:not (:not X))` as its dual.

## Surface

`(:wat::rete::not (:FactType <clause>…))` — a condition in `:when`, after the conditions that bind the
`?vars` it shares:

```
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))
   (:wat::rete::not (:ops::Maintenance (?loc <- :location)))]   ;; no maintenance at ?loc
  :then
  (:wat::rete::insert (:alert::Unattended ?loc)))
```

A token from the prior conditions passes the NegationNode iff **no** fact matches the negated condition
*for that token's bindings* (the shared `?loc` must agree).

## The scope decision (four-questions, chosen — Option A)

**Pure replay dissolves the "two-sided delta."** Clara's negation needs two-sided delta logic because it
is *incrementally streaming* (facts assert into a long-running session, so a later assert must retract a
previously-passed token). Our engine is **pure replay** (R5/P4c): within a fire the facts are fixed, so
the negated alpha-memory is fixed → negation is a **one-sided filter** (pass a token iff no compatible
element in the negated alpha-memory); across fires, replay recomputes (TM falls out, same as P4c). The
two-sided delta is *dissolved*, not skipped — the P4c move applied to negation.

**One node, not two.** Clara splits `NegationNode` (no cross-refs) and `NegationWithJoinFilterNode`
(cross-refs parent bindings, the "hardest node"). Our `token-element-compatible?` (the join's shared-var
agreement check) IS the join-filter, so **one `NegationNode`** covers both: pass the token iff zero
compatible elements in the negated alpha-memory. A deviate-UP collapse.

**Correct for base-fact negation (the dominant case).** Banked: stratified negation over *derived* facts
(the DESIGN's own captured build concern, `DESIGN.md:409` — the negated set changing across cascade rounds)
+ `:exists` = `(:not (:not X))` sugar + a *leading* `:not` (a `:not` as the first condition, with no left
token stream — needs a synthetic root token; uncommon).

## Mechanism — mirror the hash-join, inverted

A `NegationNode` is left (tokens) + right (the negated condition's alpha-memory), exactly like a
HashJoinNode — but instead of producing an extended token per compatible element, it passes the
**un-extended** token iff there are **zero** compatible elements:

- **compile-condition** (new top-branch, beside the `where`-branch): on `(:wat::rete::not <inner>)` →
  `find-or-mint-alpha` for `<inner>` (so the alpha pass populates its memory) → mint a `NegationNode`
  carrying that alpha id → wire `parent → negation` → advance parent = negation-id. (No fence: a negated
  *pattern* is data, not an expr; the inner condition's constraints are the existing pure operand grammar.)
- **fire** (the filter pass): for each `NegationNode`, for each token in `beta[parent]`, scan
  `alpha-memory[negated-alpha-id]`; pass the token (append un-extended to `beta[neg-id]`) iff **no** element
  is `token-element-compatible?`.

**Unify the filter pass.** 6b-ii's `test-pass` and this negation filter both filter `beta[parent]` →
`beta[self]`. Generalize the single fire-once filter fold to dispatch by node kind (TestNode → eval-test
filter; NegationNode → empty-negated-memory filter), folding over node-ids in **topological order** so any
interleaving of `where`/`:not` in the condition chain is correct (each filter reads its parent's beta,
populated earlier in the same fold). This replaces the standalone test-pass with a `filter-pass`.

## Decomposition

- **7-a — NegationNode in the ORACLE** (`rete.wat`): the `NegationNode` record + `Node` variant +
  `node-children` arm + `compile-condition` `:not`-branch + the unified `filter-pass` (test + negation) in
  `fire-once`. Probed via `fire-rules-spec`: `:not` passes when absent, blocks when present-matching,
  passes when present-but-different-binding (the shared-var join-filter).
- **7-b — NegationNode in the NATIVE kernel** + the differential (native==oracle on a `:not` rule).
  Mirror the native test-pass (6b-ii-b) — the negation filter over `wm.beta`/`wm.alpha` in
  `fire_fixpoint_delta`.

## Out of scope = rejected (banked, named)

- **Stratified negation over derived facts** — the DESIGN.md:409 build concern; needs the network to
  propagate only when the negated input is stable. Banked `7-strat`.
- **`:exists` = `(:not (:not X))`** — sugar; banked `7-exists`.
- **Leading `:not`** (first condition, no left tokens) — needs a synthetic root token; banked.

## Files (7-a — oracle)

- `wat/rete.wat` — `NegationNode` record + `Node` variant + `node-children` arm + `compile-condition`
  `:not`-branch + the unified `filter-pass` (generalize the 6b-ii test-pass) in `fire-once`.
- `tests/probe_arc278_7a_negation_oracle.rs` — the RED probe.

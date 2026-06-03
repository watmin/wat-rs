# Arc 248 — INSCRIPTION — the generative-macro comprehension

**Closed 2026-06-04. Absorbed into arc 237's death.**

## What it set out to do

248 opened to build the tool the equality-consolidation plan needed: a **macro that generates `=`/`not=`'s per-type defclause clauses** from a type-list — so equality could "join the one mechanism" without ~22 hand-written ceremonial clauses. wat's `defmacro` was quasiquote-only: it could *splice* a list but not *map* a sub-template over one. 248 was to add the mapping.

## What shipped — 248.1 (`c8280343`)

The **bounded `for`-comprehension**: `,@(:wat::core::for [x xs] tmpl)` maps a sub-template over a finite list at macro-expansion time and splices the results. Map, not eval — no recursion, no conditionals, no expansion-time computation. Hygiene reuses the existing sets-of-scopes (per-iteration binding cloned from the original; binder reached via explicit unquote). Probe 3/0/0, lib 895/0/1. Verified by a hard read of the `walk_template` diff, not the agent's report.

## What was absorbed — 248.2

248.2 was to be "equality → macro-generated defclause." It was never built, and that is the *correct* outcome. The dig (see `docs/DISPATCH.md`) reversed the plan: **equality is a relational intrinsic, not a clause** — the clause matcher checks each argument against a fixed named type independently and never unifies arg0's type with arg1's, while equality *is* that cross-argument unification. A finite clause list (generated or hand-written) would regress record/composite/user-type equality. So the consolidation target dissolved: equality consolidates onto the **intrinsic** mechanism (alongside collections), and `infer_equality`/`eval_eq` were already correct. **248.2 is cancelled, not deferred** — the work it would have done is unnecessary.

## What survives

The `for`-comprehension was built for a plan that reversed — but it is a **general per-Type-boilerplate generator**, kept in the substrate. The detour was not waste: it produced a real, hygienic tool, and the dig it forced produced the partition rule. FM-11 clean: 248.1 DONE, 248.2 affirmatively cancelled.

Scored, with its parent, on the killing floor (arc 237's INSCRIPTION).

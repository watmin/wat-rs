# DESIGN — Stone 8-custom: custom accumulators (any pure∧det fold fn over the gather)

## What + why
The accumulator slot today accepts only the 8 built-in folds (`:wat::rete::acc::count` … `group-by`). This
lets it accept **any user-supplied fold fn** — `(?result <- (my-fn ?v) :from (…))` where `my-fn` is a wat fn
`(PV<T>) -> R`. **This is the arbitrary-aggregate unlock** — percentiles (p95/p99), stddev/variance, top-k,
mode, weighted averages — the anomaly-domain aggregates the 8 built-ins can't express. The realization
(earlier this session): the 8 built-ins are *already* pure folds over the gather; custom = let the slot take
one more, fenced. Stone 8 built all the machinery (AccumulateNode + gather + the 6a fence); this is the
generalization.

## The one contract decision: gate on the EXISTING 6a fence (`pure ∧ deterministic`); `total?` is separate
A custom accumulator fn must be **pure ∧ deterministic** — the same fence `where`/`:test` already use
(`is_pure_expr`/`is_deterministic_expr`, 6a). Determinism is load-bearing (a random/clock fold breaks replay +
the differential). **`total?` (termination) is NOT bundled here** — a non-total fold only hangs the local
single-user engine; it becomes a *security* requirement only for untrusted multi-tenant (the service horizon,
where `total?` lives as the resource-safety axis). The percentiles/stddev/top-k fns are total by nature, so
they ship fine on `pure∧det` now. (When multi-tenant arrives, add the `total?` axis — a separate item.)

## Semantics
- Dispatch (both oracle `accumulate-pass-for-token` and native `accumulate_value`): the acc-form head keyword.
  **Known built-in head → the existing fast-path fold. Unknown head → it is a user fn name → evaluate that fn
  with the gathered values as its argument**, reusing the same fn-eval path `eval-test` uses (build a child
  env, eval the call). The result becomes the bound `?result`.
- **Gather shape (v1): value-folds** — the bound `?var` values collected into a `PV<T>` (same as `sum`/`distinct`
  gather). The fn signature is `(fn [xs <- PV<T>] -> R)`. (Fact-folds — a fn over the whole elements, for
  argmax-the-fact — are item 2 `returns-the-fact`; v1 custom is value-folds.)
- **Compile fence:** in `compile-condition`'s accumulate-branch, when the head is NOT a built-in, assert the
  user fn is `pure ∧ deterministic` (resolve the fn, run the 6a fence on its body); else raise at compile —
  exactly as `where` does. A built-in head skips the fence (already trusted).
- The 8 built-ins remain the **fast-path / standard library**; custom is the `other` arm, no longer a panic.

## Scope — one strike (oracle + native + differential)
Generalize the dispatch in both impls + the compile fence; differential `native == oracle` on a custom-fold
rule. Out of scope (separate queue items, NOT here): `returns-the-fact` (item 2), field shorthand (item 3),
`acc/` alias (item 4), `total?` resource-safety axis (horizon), fact-folds.

## Files
- `wat/rete.wat` — `accumulate-pass-for-token`: the unknown-head arm evals the user fn over the gathered PV;
  `compile-condition` accumulate-branch: fence the user fn (`pure? ∧ deterministic?`) when head ∉ built-ins.
- `src/rete/kernel/fire/acc.rs` — `accumulate_value`: the `other` arm evals the user fn over `gathered`
  (reuse the eval path; needs `sym`/env threaded in like `eval_test_core`).
- `tests/probe_arc278_8custom_native_differential.rs` — RED probe: a user fold fn (e.g. a simple `range`/`max`
  or a stddev-shaped fold) as an accumulator; native == oracle; + the fence rejects an impure fold at compile.

## Done = green
The custom-accumulator differential (native == oracle) + the impure-fold-rejected-at-compile assertion, AND
no regressions (8-a/8-b/7-exists/7a/7b/northstar/lib 941/36).

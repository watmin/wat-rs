# DESIGN — STONE A-2-ii-b: `sort$native` imposes what it declares

> The stone the whole A-2 chain was for. Builder ruled the imposition on 2026-08-30; every
> prerequisite has since shipped.

## Why now, and not five stones ago

The imposition was ruled, then **refuted as scoped** — the classifier could not see through a
closure, so the gate would have refused every `sort-by` caller. Five stones fixed that, in order:

```
A-2-i        the classifier may hold an environment
A-2-ii-a     a resolved name gets the same doors as a head
A-2-ii-b-0   Option/expect · Record/field-at · type  homed and ruled
A-2-ii-b-1   Some · Ok · Err  homed and ruled     -> the accessor finally classifies pure
(heresy)     the bare-symbol shorthand dies at all three doors
```

**All three real caller shapes now pass, measured:**

| caller | shape | `pure?` | `deterministic?` |
|---|---|---|---|
| `wat/query/mem.wat:136,163` | `sort-by` with a bare record **accessor** as keyfn | ✅ true | ✅ true |
| `wat/bracket.wat:783` | `sort-by` with an inline `fn` over a pure intrinsic | ✅ true | ✅ true |
| `sort/1` | the default `<` comparator | ✅ true | ✅ true |
| *the one that must be refused* | `(fn [a b] (do (println …) (< a b)))` | ⛔ **false** | — |

## THE ONE CONTRACT DECISION — pinned

**The gate and the declaration ship in ONE stone, because a declaration the door does not enforce is
exactly the lie this arc exists to find.** `sort$native` may declare `@Purity Pure` only once its
door refuses an impure comparator; until then `Effectful` is the honest label — and that was
*measured*, not assumed: an effectful comparator fires **4 side effects on a 3-element vector**
(`255-probe-can-a-user-make-sort-effectful.wat`).

This is a principled coupling, not a convenience bundle: split them and the first half ships a
declaration that is false, or the second half ships a gate nothing declares.

## What ships

1. **The gate**, in `eval_vec_sort_by` (`src/collection/transform.rs`): after the comparator is
   evaluated and destructured to `Value::wat__core__fn(f)`, and **before any comparison runs**,
   classify `f` against `Axis::Pure` and `Axis::Deterministic` — using the environment-carrying
   classifier with `ClassifyCtx::Runtime(f.closed_env)`, exactly as `src/freeze.rs:803` imposes on
   sigma fns.
   ⛔ **Refuse before sorting, never during.** A refusal mid-sort would have already run the
   caller's comparator on some pairs — the effects this gate exists to prevent.
2. **The homing**: `#[wat_intrinsic(":wat::core::sort$native")]` in `src/intrinsic/collection.rs`,
   a thin delegate over the existing `eval_vec_sort_by`. Its literal dispatch arm comes out; its
   `KNOWN_UNREVIEWED` row and its `macros/eval.rs` expand-time entry come out (both now derive).

## The rulings

| axis | value | ground |
|---|---|---|
| `@Purity` | **Pure** | true *because the door imposes it*, not by assertion |
| `@Determinism` | **Deterministic** | same |
| `@Total` | **Total** | measured on its own merits: a pathological comparator returns a scrambled well-formed vector, exit 0, no panic |
| `@ExpandTime` | Legal | pure ∧ deterministic, no state read |

⛔ **`Total` is NOT imposed on the comparator** — the ruling's corollary. Sort is total regardless;
a comparator that *raises* just makes the sort raise, which is ordinary propagation. Imposing it
would refuse every accessor key (they are `Partial` via `Option/expect`) **for no defect**.

## ★ A PREDICTION, and it is falsifiable

Unlike the last two homings, this one should **not** trip `checker_skip_debt_is_named_and_frozen`:
`sort$native` **has** an `env.register()` TypeScheme (`src/check.rs:20322`), so `check_env.get`
resolves and the doc gate can compare against it. **No `FROZEN_CHECKER_DEBT_LEDGER` row is expected.**
If one is needed, the measurement is wrong and that is a finding — not a row to add quietly.

## Out of scope = REJECTED (not deferred)

- **`map · mapv · filter · foldl`** — the W7 family. This stone proves the mechanism on one verb;
  extending it to the HOFs is a language ruling about whether an effectful `map` callback is
  legitimate, and that is the builder's, separately.
- **`src/freeze.rs:803` opting into the environment-carrying classifier** — it has the same blind
  spot and A-2-i made the fix available; changing a startup gate's verdicts is its own stone.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **b** gate + declaration in one stone | YES | YES | YES | YES | ✅ **ADMITTED** |
| declare `Pure`, add the gate later | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| gate now, declare `Unreviewed` for now | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| impose `Total` as well | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **declare-first Honest? NO** — ships a `Pure` that a user can falsify in one line, which is the
  measured status quo.
- **gate-first-declare-later Honest? NO** — `Unreviewed` would mean *nobody looked*, when the door
  next to it enforces the answer. That is the "did not look" lie in reverse.
- **impose-`Total` Honest? NO** — a demand with no defect behind it, refusing every accessor key.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ an effectful comparator is REFUSED | `(sort$native (fn [a b] (do (println …) (< a b))) …)` | a located runtime error naming the offending head |
| ★ refused with **no effects emitted** | same | **zero** `println` output before the error |
| `query/mem.wat`'s accessor key still works | `sort-by :Row/sk` over a vector | sorted, unchanged |
| `bracket.wat`'s inline-fn key still works | `sort-by (fn [pr] (first pr))` | sorted, unchanged |
| `sort/1` and `sort/2` still work | the public surface | `[1 2 3]` · `[3 2 1]` |
| the registry answers | `lookup_entry(":wat::core::sort$native")` | `Some`, `Purity::Pure` |
| both hand-lists shrink | `KNOWN_UNREVIEWED`, `macros/eval.rs` | one row each, gone |
| the prediction holds | `FROZEN_CHECKER_DEBT_LEDGER` | **unchanged** |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

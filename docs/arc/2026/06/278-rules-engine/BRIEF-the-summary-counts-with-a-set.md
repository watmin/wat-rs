# BRIEF — the summary counts with a set

## The work, in one paragraph

`:fanout::summarize` folds over every outcome — 8000 at n=2000 — building two `HashMap`s with
`:wat::hashmap::assoc`, which clones the whole map on each insert. `id-map` grows to 8000, so the harness
carries an O(N²) term. Both maps are **sets wearing map clothes**: every value is `true` and the only use
is a count. Swap them to `:wat::set::` and read `distinct` from `:wat::set::length`.

## Read in order

1. **`wat-scripts/fanout/circuit.wat:2090-2110`** — `:fanout::summarize`. The two folds, the `distinct`
   count from `hashmap::keys`, and `dup = total - distinct`. **This is the only region you change.**
2. **`src/collection/eval.rs:360-370`** — `hashmap::assoc`'s `(**m).clone()`, the cost you are removing.
3. **`src/collection/eval.rs:388-410`** — `persistentset_conj_inner`, `(**s).insert(item.clone())`, the
   replacement's actual implementation.
4. **`wat-scripts/queue/sqs.wat`** — search `seen-ids`. **The worked precedent**, landed in `0e026d2de`:
   a `PersistentSet` threaded through a fold accumulator, including the type ascriptions the closure
   signature needs. Its BRIEF named three edits and the real count was six, for exactly that reason.
5. **`src/intrinsic/set.rs`** — the five verbs. There is no `keys`; `length` reads the size.

## Implementation sketch

```wat
id-set (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::PersistentSet :- [:wat::core::String])
                          o   <- :fanout::Outcome]
           -> (:wat::core::PersistentSet :- [:wat::core::String])
           (:wat::set::conj acc (:fanout::key-of o)))
         (:wat::core::PersistentSet :- [:wat::core::String])
         outs)
;; w-set the same, keyed on (:fanout::Outcome/worker o)
distinct (:wat::set::length id-set)
workers  (:wat::set::length w-set)
```

`dup (:wat::core::- total distinct)` is unchanged.

## Blast radius

`wat-scripts/fanout/circuit.wat`, confined to `:fanout::summarize`. **No `src/`.** No other `.wat`.

⚠ The four `:user::` scenario deftests at `:2644 :2732 :2836 :2893` keep their `:wat::hashmap::assoc` **by
decision** — tiny inputs, unmeasurable cost. A file-wide `grep -c hashmap` will not read zero, and should
not. Row 3 scopes the check to `summarize`.

## STOP triggers

**STOP-1** — if `distinct` or `dup` changes by even one, **STOP**. `distinct` is how the circuit detects
loss and duplication; a shift means the key set changed, which is a correctness break dressed as an
optimisation.

**STOP-2** — do **not** claim a speedup. The O(N²) term is ~small at n=2000 and below fill's run-to-run
spread. Report `total` and the phase sum and their difference; let the numbers speak. A reported speedup
here has measured variance.

**STOP-3** — do **not** touch the four scenario deftests, `wat/`, the queue, the topic, admission, or the
cap.

**STOP-4** — if `:wat::set::` lacks something you need, **STOP and name it.** This is the type's second
production caller; a gap it exposes is a finding about the type, not a reason to keep `:wat::hashmap::`.

**STOP-5** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

One n=2000 `vis-ms=1000` run on a quiet box (state the load) reporting **`distinct=8000` and `dup=0`** —
that is the gate — plus `workers` in the 8–12 range, and **`total` beside the sum of
`setup+fill+arm+drain+collect+stop`, with the difference**, against the ~560 ms baseline. `grep` finds no
`:wat::hashmap::` inside `:fanout::summarize`. `every_wat_scripts_file_loads` PASS. Floor Summary reads
5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

**Write the SCORE to `docs/arc/2026/06/278-rules-engine/SCORE-the-summary-counts-with-a-set.md`** in the
shape of the neighbouring SCORE files: the floor's Summary line verbatim, a row-by-row table for all nine
rows, the measured numbers beside the baseline, and the blast radius. **Do not commit.**

Expect the edit count to exceed the sketch — the accumulator type appears in the closure parameter, the
closure return, and the fold's initial value. Report how many sites you actually touched.

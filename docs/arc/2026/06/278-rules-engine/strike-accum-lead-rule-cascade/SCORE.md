# SCORE — the fourth cell agrees; L2-1 did not leak into Busy rows

Leading accumulate in a RULE, with an inert cascade. `:derived` is `[3 1000000000000003]` at
depth 1 and depth 3, native and oracle. Clara matches. Count is `anchors` = 2, constant.
STOP-1 did not fire. This is DESIGN outcome 2 for **this cell at these sizes**, not a licence
to forget `accumulate.rs:138`. No `src/` was touched.

## Scorecard

| # | result |
|---|---|
| 1 ★ axis runs | **HOLD.** `#grid/Result` at `[3 2 3]` and `[3 2 1]`. |
| 2 ★ independent of depth | **HOLD.** Depth 1 and depth 3, both engines: `:derived [3 1000000000000003]`. Quoted below. |
| 3 ★ native == oracle | **HOLD.** Byte-identical at both depths. |
| 4 ★ Clara agrees | **HOLD.** `check-grid-three-way.sh accum-lead-rule-cascade`: `clara=2 native=2 oracle=2 ALL THREE MATCH`. |
| 5 ★ count is `anchors`, constant | **HOLD.** Expected 2, derived from the header before the run — not a function of depth. |
| 6 ★ the axis can fail | **HOLD.** Temporary mutation: `:derived` included unread Link levels. Port check: engines AGREE on 5 elems where the shape predicts 2. Quoted below. Restored. Not re-run. |
| 7 ★ floor | **HOLD.** `Summary [ 487.682s] 5475 tests run: 5475 passed (3 slow), 22 skipped`. `.floor/2026-09-07T12-29-07Z/`. Count unchanged: walking tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 no src/ | **HOLD.** `accumulate.rs`, `filter.rs`, `delta.rs`, and the three covered axes untouched. |

★ load-bearing. **Row 2 is the L2-1 drive. A green here does not close the unguarded re-seed; it says this cell did not leak rounds into Busy.**

## Depth independence (items=3, anchors=2)

```
#grid/Result {:axis "accum-lead-rule-cascade" :size [3 2 1] :derived [3 1000000000000003] … :oracle-derived [3 1000000000000003] …}
#grid/Result {:axis "accum-lead-rule-cascade" :size [3 2 3] :derived [3 1000000000000003] … :oracle-derived [3 1000000000000003] …}
```

`enc(0,3)` and `enc(1,3)` — two Busy, n=count(Reading)=3. Clara:

```
accum-lead-rule-cascade clara=2    native=2    oracle=2     ALL THREE MATCH
```

Sizes actually run: `[3 2 1]`, `[3 2 3]` (hand), `[3 2 3]` (three-way / port_check), `[2 1 2]` (liveness). Not a sweep.

## Row 6 — quoted red, restored

Witness temporarily concatenated Link levels > 0 into `:derived`. At `[3 2 3]`:

```
:derived [1 2 3 3 1000000000000003]   ; 5 elems (3 Link + 2 Busy)
```

Port check (captured, not re-run):

```
  accum-lead-rule-cascade (size [3, 2, 3]): native and oracle AGREE, but on 5 element(s) where this axis's own shape predicts 2. The engines are not implicated — either the workload changed (update the derivation, do not update the number) or the run is not the one this row describes.
```

That is the constant-count instrument naming extra cascade rows. Restored: Busy-only, `[3 1000000000000003]`.

## What this does not say

`accumulate.rs:138` still re-seeds an empty token every round with no `leading_emitted` guard.
This cell did not turn that into extra Busy rows at depth 1 vs 3, items=3, anchors=2. The
matrix header still says a fix for one cell did not reach the other — this axis is the watch
on the fourth cell, not a proof the seed is unreachable everywhere.

## Landing

`accum-lead-rule-cascade.{wat,clj}`, three registration rows. Static `.clj`, no `gen-`. Do not
commit unless asked.

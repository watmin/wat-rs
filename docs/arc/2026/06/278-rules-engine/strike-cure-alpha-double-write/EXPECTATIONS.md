# EXPECTATIONS — D7's cure

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the repro agrees with the oracle | **`native=2 oracle=3`** | **`native=3 oracle=3`** |
| 2 | ★ the invariant holds | two writers can hit one `aid` | one writer per `aid` per pass, by the chosen shape |
| 3 | the width control is unchanged | `wide=3 narrow=3` | identical — STOP-4 |
| 4 | a differential gate exists on the floor | **none over a parametric record** | present, and RED under mutation 1 |
| 5 | the decision is read, not decorative | — | mutation 2 |
| 6 | element ordering preserved | `d_alpha` indexes into the vec | unchanged, or STOP-1 |
| 7 | hot-path cost stated | — | measured and named |
| 8 | `leaf_occ` NOT used as the gate | it is blind (C16) | not used |
| 9 | floor | 5336/5336 | ≥ 5,336, zero FAIL |
| 10 | lints / clippy | 210/210, rc=0 | green, silent |

## What would make this strike a failure even if every test passes

**A cure that narrows batching to nothing.** Making *no* class batchable satisfies the invariant
trivially, turns rows 1 and 2 green, and silently deletes the occupancy fast path. **Row 3 and
mutation 2 are what catch it** — the width control must still batch.

**And a gate that only reproduces this one program.** The defect class is *an erasure seam that makes
one class's instances differ in packability*. A differential pinned to `Box[i64]`/`Box[String]`
specifically will not see the next one. Gate the property, not the fixture.

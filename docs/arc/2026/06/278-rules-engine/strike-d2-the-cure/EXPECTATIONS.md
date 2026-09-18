# EXPECTATIONS — the cure

> ⚠ **Done is the banked test un-ignored and GREEN.** Nothing less closes this; the defect is live in
> shipped behaviour.

## ⛔ NO PINNED TEST COUNT

**The floor must be green with `right_index_counter_tracks_its_bucket_population` RUNNING, not
skipped.**

## The scorecard — pre-values driven at HEAD `72b894ccb`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the invariant holds | ⛔ **J6 12 vs 18, J11 6 vs 12**, persisting to fixpoint | every join, every round, `mark == Σ bucket lengths` |
| 2 | ★ the acceptance test runs | banked `#[ignore]` | **un-ignored and GREEN**; the `#[ignore]` line gone, not commented |
| 3 | ★ the bypass is unrepresentable | 3 appenders, 1 maintainer | one insertion verb, **no path that appends without advancing** — or STOP-2 fired and was reported |
| 4 | the cure is what holds it | — | mutation 2: re-introducing a bypass REDs |
| 5 | the verb really maintains | — | mutation 3: deleting its mark update REDs |
| 6 | facts do not move | `seen_insert` already dedups | **zero derived-fact change on any axis** |
| 7 | controls still discriminate | J4, J9 clean | still clean — a cure that makes everything trivially equal is not a cure |
| 8 | floor / lints / clippy | lints **265**, clippy rc=0 | floor green **with the test running**, 0 FAIL, lints ≥ 265, rc=0 |

## Runtime prediction

**70–110 minutes.** ⚠ The last strike predicted 50–80 and took ~3.5 h on seven release builds; budget
the rebuilds, not the edit.

## Trap doors named in advance

- **⛔ `right_idx[J].len()` IS THE BUCKET COUNT.** The value is a `JoinKeyMap`. The invariant is over
  **Σ bucket lengths**. Getting this wrong reports a false positive on every multi-key join.
- **The convention rung is not the cure.** Bumping the counter at two call sites leaves the third
  writer free to appear — this defect already survived `partire` for exactly that reason. STOP-2.
- **A cure that makes the controls trivially equal is not a cure.** Row 7.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**Leaving the test banked.** The whole point is that a live defect was made invisible by an
`#[ignore]` borrowed from an idiom meant for unbuilt features. If the cure lands and the test stays
skipped, nothing has been proven and the floor is green over the same hole.

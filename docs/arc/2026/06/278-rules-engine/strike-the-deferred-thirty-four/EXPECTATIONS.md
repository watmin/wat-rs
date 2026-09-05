# EXPECTATIONS — the fence empties

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,418 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `5aa25e0c4`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the fence empties | **34 `DEFERRED` rows** | **0**, and the const deleted |
| 2 | ★ no re-point on a name match alone | 12 rows have a same-named file; **≥1 is a false target** (`kernel/tests.rs` → `src/macros/tests.rs`) | every re-point carries evidence the target is the **same artifact**; every judged-not-same row is **named** |
| 3 | ★ cures are gated, not just deleted | — | mutation 1: re-introducing a cured citation REDs |
| 4 | the const cannot rot | — | mutation 2: a row whose citation is cured REDs as unmatched |
| 5 | removal is earned | — | mutation 3: emptying early REDs |
| 6 | golden line pins respected | 4 goldens pin `wat/core.wat` | edits line-count-neutral **or** goldens regenerated deliberately; **the pinned set determined, not assumed** |
| 7 | prose left true | — | any sentence that becomes false when its path goes is **reported**, not quietly reworded |
| 8 | floor / lints / clippy | **`5418 tests run: 5418 passed, 21 skipped`** (445.2 s, 0 FAIL), lints **265**, clippy rc=0 | ≥ 5418 + arms, 0 FAIL, lints ≥ 265, rc=0 |

## Runtime prediction

**60–90 minutes.** The edits are small; **the verification is the work** — 34 sentences, each needing
a judgement about what its citation was for.

## Trap doors named in advance

- **⛔ THE PLAUSIBLE TARGET IS THE DANGER.** A dead link announces itself; a confident wrong one does
  not. Row 2.
- **A comment-only `.wat` edit can turn the floor red** — F2-e's rider proved it on `wat/core.wat`.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.
- **Assert the row count before reasoning from a split.** The orchestrator's first parse of this const
  said 14/28 because it matched tuples in the gate's own unit tests.

## What would make this strike a failure even if every test passes

**Re-pointing a citation at a file that merely shares a name.** That converts a defect the gate can
see into one it cannot — the gate checks that a path *exists*, not that it is the *right* path. Row 2
is the strike, and the list of rows judged not-the-same is worth more than the count of rows cured.

**And an empty const with the prose left lying.** If a sentence asserted something that died with its
file, deleting the path makes the gate green and leaves the false claim. Row 7.

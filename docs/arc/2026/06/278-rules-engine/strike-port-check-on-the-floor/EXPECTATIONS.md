# EXPECTATIONS — C9's port half

> ⚠ **This strike buys the PORT pairing only.** `oracle` vs `clara` — *"the SPEC is wrong"* — still
> needs the JVM and stays open under C9. A report claiming C9 closed is wrong.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,375 plus every arm you drive.** An equality would cap coverage downward
while looking like rigour. Row 10's pre-values are the floor you must clear, never the number you
must reproduce.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the port check runs on the floor | **never run, in 23 grids** | a gate, all axes |
| 2 | ★ the corpus can express D7's shape | **0 of 185 `defrecord`s parametric** | a parametric axis present |
| 3 | ★ the new axis REDs on D7's defect | — | mutation 1, both sets named |
| 4 | the gate reads the ORACLE column | — | mutation 2 |
| 5 | failure names both sets | — | not a count — D7 was right-sized and wrong-valued |
| 6 | `fanout` covered | ⚠ **the old pre-value here was FALSE** — it said *"emits `#fan/QuerySplit`"* as if instead of `#grid/Result`; it emits **both**, `QuerySplit` first. Re-driven at HEAD: **400 derived at `[500]`** | covered, not excluded |
| 6b | ★ **every set non-empty** | **`fanout` at `[20 5]`, `[100 20]`, `[50 10]` all print `match` on `[] == []`** (driven) | the gate REFUSES a vacuous axis — equality is satisfied by absence |
| 7 | all axes green at HEAD | **11/11 match, 5.7s, re-driven at `daa92c3b0`** — 49·25·20·25·200·75·5·20·50·200·400 elements | still 11/11 (+ the new axis) |
| 8 | runtime | 5s for 11 | ≲ 60s — STOP-4 |
| 9 | no `src/` change | — | zero diff under `src/` |
| 10 | floor / lints / clippy | ⛔ **the old pre-values `5345, 210/210` were STALE by 30 tests and 18 lints** — drawn at `3144f9123`, three `src/` commits ago. **Measured at HEAD `daa92c3b0`, 427.7s: `5375 tests run: 5375 passed, 21 skipped`, 0 FAIL rows, `wat::lint` 228 PASS** | **≥ 5375 plus every arm you drive**, zero FAIL rows, lints ≥ 228, clippy rc=0 |

## What would make this strike a failure even if every test passes

**A vacuous axis counted as a pass.** Row 6b is not bookkeeping: an empty-vs-empty comparison prints `match`, and this strike found one live in the corpus on the first drive. A gate that cannot tell *"they agree"* from *"there is nothing to disagree about"* is C16 rebuilt.

**Landing the gate without the parametric axis.** It would be green on a corpus that *cannot express*
the defect this arc just spent a day on — a differential whose corpus has a hole shaped like the bug,
now with a green light over it. Rows 2 and 3 are the whole strike; row 1 alone is theatre.

**And a gate that compares counts.** D7 produced a *right-sized wrong answer* in one of its arms
(`d_alpha` indexing elements that moved under it). A cardinality check would have passed it.

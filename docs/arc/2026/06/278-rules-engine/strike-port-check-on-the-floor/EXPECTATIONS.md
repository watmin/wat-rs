# EXPECTATIONS — C9's port half

> ⚠ **This strike buys the PORT pairing only.** `oracle` vs `clara` — *"the SPEC is wrong"* — still
> needs the JVM and stays open under C9. A report claiming C9 closed is wrong.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the port check runs on the floor | **never run, in 23 grids** | a gate, all axes |
| 2 | ★ the corpus can express D7's shape | **0 of 185 `defrecord`s parametric** | a parametric axis present |
| 3 | ★ the new axis REDs on D7's defect | — | mutation 1, both sets named |
| 4 | the gate reads the ORACLE column | — | mutation 2 |
| 5 | failure names both sets | — | not a count — D7 was right-sized and wrong-valued |
| 6 | `fanout` covered or excluded with a reason | emits `#fan/QuerySplit` | stated either way |
| 7 | all axes green at HEAD | **11/11 match, 5s** | still 11/11 (+ the new axis) |
| 8 | runtime | 5s for 11 | ≲ 60s — STOP-4 |
| 9 | no `src/` change | — | zero diff under `src/` |
| 10 | floor / lints / clippy | 5345, 210/210, rc=0 | green |

## What would make this strike a failure even if every test passes

**Landing the gate without the parametric axis.** It would be green on a corpus that *cannot express*
the defect this arc just spent a day on — a differential whose corpus has a hole shaped like the bug,
now with a green light over it. Rows 2 and 3 are the whole strike; row 1 alone is theatre.

**And a gate that compares counts.** D7 produced a *right-sized wrong answer* in one of its arms
(`d_alpha` indexing elements that moved under it). A cardinality check would have passed it.

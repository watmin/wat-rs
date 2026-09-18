# EXPECTATIONS — the fast filter must agree with the reference filter

> ⚠ **This strike PROVES the current behaviour. It changes no engine logic.** The scan optimisation
> is a separate, later strike — an optimisation to a path with no differential is a change nobody can
> prove safe.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,407 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `268bd868b`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the branch pair has a differential | ⛔ **none** — the tree path skips evaluations and pushes unevaluated facts, unchecked | a gate comparing derived **fact sets** |
| 2 | ★ obligation 1 is guarded | ⛔ unchecked: `covers && !proven && !maybe` ⟹ test is false | mutation 1 REDs, naming the **dropped** fact |
| 3 | ★ obligation 2 is guarded | ⛔ unchecked: `proven && is_pure_cmp` ⟹ test is true | mutation 2 REDs, naming the **invented** fact |
| 4 | ★ the corpus can express the defect | `node-share [50 200]`: **reuse 200, evals 0** — every decision made by the tree | the tree-firing population **measured and listed**, not assumed |
| 5 | sets, never counts | — | failure prints both sets + symmetric difference. **D7 was right-sized and wrong-valued** |
| 6 | not vacuous | — | mutation 3: an empty population FAILS |
| 7 | the engine is unchanged | — | **zero diff in `dispatch_where_tests`**; any `src/` change is visibility only, and named |
| 8 | floor / lints / clippy | **`5407 tests run: 5407 passed, 21 skipped`** (439.3 s, 0 FAIL), lints **258**, clippy rc=0 | ≥ 5407 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**60–90 minutes**, plus release rebuilds for the mutation matrix — **budget four**. The last two
strikes overran their estimates on exactly this.

## Trap doors named in advance

- **⛔ A RED HERE IS A LIVE SOUNDNESS BUG, NOT A TEST PROBLEM.** If the differential fires at HEAD,
  the filter is dropping or inventing derived facts today. Capture both sets verbatim and STOP.
- **A corpus that never fires the tree proves nothing.** C9's gate was green over a corpus with a hole
  shaped exactly like the bug it missed. Row 4 exists because of it.
- **Counts pass what sets catch.** D7's native answer was the right cardinality with the wrong
  elements.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**A differential that catches one obligation and not the other.** Skipping an evaluation that should
have passed drops a fact; pushing an unevaluated fact that should have failed invents one. These are
different bugs on different arms, and mutations 1 and 2 exist to prove both are reachable. **One
mutation cannot prove a two-arm gate.**

**And coverage assumed rather than measured.** If the gate runs over fixtures where `use_tree` is
false, it is comparing the reference branch against itself — `X == X`, which this arc found in a
landed gate two days ago. Row 4 is what prevents it.

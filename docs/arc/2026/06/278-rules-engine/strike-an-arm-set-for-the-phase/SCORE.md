# SCORE — the phase has an arm set; the number that justified the strike was wrong by 16x

> **Written after the orchestrator's own weighing.** The ★ correction was verifiable from a
> doc-comment sitting on the function the whole strike was about.

## The scorecard, graded

| # | required | result |
|---|---|---|
| 1 ★ | the taken branch has arms | ✅ **G–L**: `bind_view` → `+candidates` → `+the two HashSets` → `+the tid loop` → `+d_beta pushes`, plus **L**, the parent gather |
| 2 ★ | coverage MEASURED with spread | ✅ **89.3% median (84.2–93.5 over six)**, p25/p75 band per run; **~11% unaccounted and NAMED** |
| 3 ★ | coverage never ARRANGED | ✅ **refused as an assertion** — the remainder is the size of the scatter, so any interesting bound would be chosen to pass |
| 4 | pre-value re-measured at six | ⚠ **STOP-4 fired** — `659–697` (median 670.5) vs C6's `684–734` (median 695.5). Same code. **Nothing was built on the difference** |
| 5 | old arms keep their meaning | ✅ relabelled as the `exec_where` branch; `F+C` printed as a **labelled non-reconstruction** |
| 6 | each arm wired to the number | ✅ mutation 1: zeroing the tid loop moved coverage **87.9% → 16.2%** |
| 7 | the two ladders distinguishable | ⚠ **half proven, half honest STOP-1** — see below |
| 8 | arms scale with the phase | ✅ **and it refuted my prediction** — see below |
| 9 | no engine change | ✅ one file; `git diff` over `fire/` and `expr_ir/` is empty |
| 10 | floor / lints / clippy | ✅ **`5407 tests run: 5407 passed, 21 skipped`** (439.3 s), **0 FAIL rows**, lints **258**, clippy rc=0 |

## ⛔⛔ THE ★ — ARM C MEASURES THE WRONG TYPE, AND IT IS THE NUMBER THE STRIKE RESTED ON

The DESIGN, the BRIEF and the orchestrator's own drive all say:

> *"Only arm C measures work the engine performs: 0.132 / 0.419 = 31.5%."*

**Arm C clones `Vec<PMap>` — persistent maps, a deep clone. The fire clones `Vec<Token>`.** And
`d_beta_from_parents`'s own doc-comment says so:

> *"`Token` is 16 B and `Copy`, so this is a **memcpy, not a deep clone**."*

Measured: the real gather (**arm L**) is **~8 µs** against arm C's **~127 µs** — **~16×**.

**The count was right and the type was wrong.** 50 clones of a 200-element vector — `dbeta:alloc` 50
and `dbeta:tokens` 10,000 confirm the cardinality exactly. `[[a-count-cannot-see-a-value-defect]]`,
and the answer was in a doc-comment on the function the strike was about, which neither C6 nor the
orchestrator read.

**Consequence: "31.5% accounted by C" was ~2%. The real unmeasured fraction at HEAD was ~98%, not
68.5%.** The strike's justification was understated by a third of the phase.

## ★★ AND THE PHASE IS ONE RUNG NOBODY HAD LOOKED AT

**J−I — the tid loop — is ~78–83% of the taken branch and ~70% of the live `filter` phase.** 10,000
(token, tid) pairs, three `HashSet<i64>` probes each, **9,800 of which reach `continue`**. It is the
only rung that scales with N×M.

**My DESIGN listed five mechanisms and gave them equal billing.** Nothing in the artifacts ranked
them, and the one that is the phase was buried mid-list.

## Mutation 3 refuted my own prediction, and the refutation is the better result

My BRIEF predicted: *"halving tokens → every arm scales and **the coverage fraction holds**."*

**The arms scale; the fraction does not** — 88% → 78.3%, because **~40–55 µs of the phase is
token-independent** (`filter_pass`'s walk over all 50 nodes, twice, once per round). Had the rider
taken my prediction as the pass condition, **it would have called a correct instrument broken.**

That measurement also *bounds* the remainder, which a holding fraction would not have.

## Mutation 2 — half proven, half correctly refused

Proven: denying `is_pure_cmp` for one tid in the **harness replica** REDs with
*"the replica took reuse=196 eval=4 … the FIRE counted reuse=200 evals=0"* — so the replica is pinned
to the branch the fire takes.

Refused, with the check rather than the assumption: making the **fire** take the `else` needs either a
hot-path edit (**C10 forbids**) or a different rule set (**the DESIGN rejects**). `WhereTree` has one
production construction site (`arm.rs:1054`), `WhereTree::empty()` appears nowhere in production, and
there is no env var or feature flag. **STOP-1 is the honest answer and it was given.**

## The rider's own RED, on its sixth drive, not re-run for green

Its first-cut monotone invariant `K >= J` failed on the **sixth consecutive** run:

```
the cumulative ladder is not monotone, so a rung's added work was optimised away and its delta is
an artifact (G=338ns H=46755ns I=56527ns J=349781ns K=345199ns L=6697ns)
```

K came in 4,582 ns **below** J while the K−J rung is only ~8,000 ns — the small rungs sit below the
instrument's resolution. It **did not re-run for green**: it removed the ungateable comparison,
recorded the red verbatim in-code with a warning not to re-add it *because it passed five times*, and
kept the table printing negative rungs rather than hiding them. It also replaced a min/max band
(outlier-driven — one scheduling stall rendered `84.9–153.3%`) with p25/p75 over a trimmed range,
raw extremes still printed.

**Five green runs would have shipped that assertion.** The sixth was taken as evidence.

## What is asserted, and what is not

**Refused:** the reconstruction. `K+L ≤ filter` is structurally necessary but unresolvable here — the
~11% remainder is about the size of the ~9-point run-to-run scatter, so any interesting bound is a
number chosen to pass. Coverage prints with its band, six samples in-code, in C6's shape.

**Asserted** (all far above noise, none a tolerance): `filter:test-reuse > 0 && filter:test-evals == 0`
(the branch identity the ladder rests on) · **replica ≡ fire** (one dispatch's reuse/eval counts equal
the whole fire's) · `dbeta:tokens == dbeta:alloc × width` from the census, not from N · zero dispatched
tids are `beta_readers` · ★ the tid loop is the majority rung, with a ~6× margin.

## Honest deltas — six more corrections to my artifacts

- **The DESIGN's pseudo-code of the taken branch is incomplete** — it omits `use_tree`, the
  `covers(tid)` guard **inside the tid loop (the dominant cost)**, the `beta_readers` probe, the
  `wm.beta` push, and two `#[cfg(test)]` `census_count` calls that are *inside* the `filter` reading
  the reconstruction divides by. **Building from my listing alone would have missed the ~80% rung.**
- **My line numbers are stale.** `fire/mod.rs:2038/:2039/:2040/:2068` — `dispatch_where_tests` starts
  at `:2010`, the `is_pure_cmp` arm is at `:2043`, and a cited `:2701` names a line in a 2094-line file.
- **EXPECTATIONS row 10** says *"≥ 5407 plus every arm you drive"* — the new arms live inside an
  existing `#[test]`, so the count is unchanged. A literal reading would have added a test to satisfy
  arithmetic that does not apply.
- **Neither artifact mentions `d_beta_from_parents`, `dbeta:alloc`/`dbeta:tokens`, or
  `arm.test_sibs`** — the three things that turn the arms' scale from a guess into a measurement, all
  present at HEAD.
- **`[[a-found-range-can-span-more-than-you-think]]` fired again, on the rider.** A python edit whose
  range ran from a comment header to an assert's closing paren **silently swallowed the ★ assert**,
  leaving prose that said *"the ★ below"* with nothing below it. Caught only because mutation 1 came
  back **green when it should have been red**. Six post-value runs had been driven without it; it was
  restored and mutation 1 re-run.
- **Runtime overran again** on release rebuilds, as the last strike did.

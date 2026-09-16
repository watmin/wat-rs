# EXPECTATIONS — C3

> ⛔ **THIS SCORECARD RECORDS INVARIANTS, NOT MILLISECONDS.** C4's scorecard pinned absolute times
> and three readings of an unchanged tree spanned ~16% on the same box. Rows below state what must
> be TRUE, not what number was seen. Timings appear only where the claim *is* about a sign.

## ⛔ NO PINNED TEST COUNT

The floor must be **≥ its current value plus the new lint's cases**, with zero FAIL rows.

## The scorecard

| # | what | state AT HEAD (verified) | required after |
|---|---|---|---|
| 1 | ★ the lint exists and resolves phase names | absent | present under `tests/lint/` |
| 2 | ★ it is RED at HEAD on exactly one name | — | RED naming `"  │  setup:seen:insert"` at `accum_cost.rs:1603`, and **no other** |
| 3 | ★ it is GREEN after piece 2 | — | green |
| 4 | the dead read is gone | `fire_ins = ...of("  │  setup:seen:insert")` at `:1603` | no reader of that name anywhere |
| 5 | the false rows are gone | table prints `insert` and `in-fire insert − S` | both absent |
| 6 | no row is negative *because a mark was missing* | `in-fire insert − S` prints **−2.55 ms** | that row does not exist; no surviving row is negative from an absent mark |
| 7 | the table states what `setup:seen` covers | doc at `:1528` claims an alloc/insert split | says coextensive with `:alloc`, allocation only, insert lives in `alpha`+`production` |
| 8 | `REQUIRED_PHASES` untouched | 25 entries, already correct | unchanged |
| 9 | engine untouched | — | zero diff under `src/rete/kernel/fire/` |
| 10 | radius | — | `accum_cost.rs` + one new `tests/lint/` file |
| 11 | lints | 196/196 | green, plus the new cases |
| 12 | clippy | rc=0 | silent |

## The mutation proofs

1. **Restore the dead read** → lint RED naming it. Proves the lint sees the defect it was built for.
2. **Typo an EXISTING mark** in a cost test → lint RED naming that one. Proves it resolves names
   generally rather than special-casing one string. *(Row 2 alone cannot distinguish these.)*
3. Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

45–70 minutes. The lint is the work; the table edit is a deletion and a doc rewrite.

## What would make this strike a failure even if every test passes

**A lint that only ever matches this one string.** Mutation 2 is the whole defence: without it, a
hard-coded check against `setup:seen:insert` passes every other row on this card and gates nothing.

**And deleting the rows without restating the premise.** The test's doc and
`DESIGN-STONE-seen-fire-context` assert an alloc/insert split that does not exist. Removing the rows
while leaving that sentence standing moves the false claim from the output into the prose, where
this arc has repeatedly shown it survives longer.

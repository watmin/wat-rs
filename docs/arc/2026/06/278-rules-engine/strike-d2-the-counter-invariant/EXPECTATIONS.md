# EXPECTATIONS — the counter invariant

> ⚠ **This strike does not fix anything.** It decides whether D2 is live or bounded, with an
> instrument. Both answers close the row; only one of them is a bug report.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,418 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `974e0d859`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the counter is inspected directly | ⛔ **never** — both prior drives were end-to-end, and `seen_insert` dedups their observable | `indexed_n[J] == right_idx[J].len()` asserted per round |
| 2 | ★ the bypass sites are proven reached | ⛔ unmeasured | `hash_join.rs:185` **and** `:298` shown executed; **mutation 2 proves it** |
| 3 | ★ a verdict either way | *"latent, not live"* on an uninstrumented premise | **LIVE** or **bounded-with-an-instrument**, stated plainly |
| 4 | the assert reads the real pair | — | mutation 1 REDs naming J, counter, length, round |
| 5 | not vacuous on the wrong shape | — | mutation 3: a single-HashJoin shape FAILS as inapplicable |
| 6 | the row's stale evidence corrected | *"not even a parameter"* — **false**; `partire` threaded it into both passes | recorded in the row |
| 7 | no engine logic change | — | probe + visibility only; **zero behaviour edits** |
| 8 | floor / lints / clippy | **`5418 / 5418`**, 0 FAIL, lints **265**, clippy rc=0 | ≥ 5418 + arms, 0 FAIL, lints ≥ 265, rc=0 |

## Runtime prediction

**50–80 minutes.** Constructing the two-HashJoin shape and proving both bypass sites fire is the work;
the assert is three lines.

## Trap doors named in advance

- **⛔ A GREEN INVARIANT OVER UNREACHED CODE IS THE VACUOUS PASS THIS ARC KEEPS FINDING** — C9's
  corpus, C16's filter, C14's counter, the `assert!(!ok)` idiom, the `X == X` gate. **Row 2 is the
  strike.**
- **The refactor moved this defect DOWN the ladder.** A missing parameter is compiler-visible; an
  unused `&mut` is a convention. Do not read "it's a parameter now" as progress.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.

## What would make this strike a failure even if every test passes

**Reporting "invariant holds" without proving the bypass sites ran.** That is the same green-over-
nothing that let D7 hide behind a filtered differential for weeks, and it would close the vigilia's
last row on exactly the defect the vigilia exists to find.

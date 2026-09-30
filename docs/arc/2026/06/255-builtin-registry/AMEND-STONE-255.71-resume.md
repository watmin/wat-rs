# AMEND — STONE 255.71: resume after the rate-limit stop

**Drawn 2026-09-30.** The first 255.71 agent, and the helper agents it spawned, were cut off on 2026-09-28 by the
Sonnet weekly limit. The brief `BRIEF-STONE-255.71-the-untyped-constructor-wall.md` stands. This records the state it
left, and one added rule.

## The state on disk (orchestrator-verified, 2026-09-30)

- Nothing is committed past the draw `75450a149`.
- **The wall is in `stash@{0}`** ("255.71 wall (src/check.rs) - stash-dance": `src/check.rs`, +65/−7). It was stashed so
  the codemod could convert with the old checker (`wat/fix.wat`'s STASH-DANCE note).
- **Untracked tests:** `tests/function/probe_stone255_71_{the_wall,template_typed,template_untyped,list_bracketless_illegal}.*`.
  `tests/function/probe_stone255_70_constructor_brackets_through_the_door.{rs,wat}` are modified (255.70's bracket-less
  regression row, which the wall makes illegal).
- **Type-table work in `scratch/`** (untracked): `255-71-census-pre.tsv` (177 sites), and per-area tables merged into
  `255-71-master.tsv` (**158 rows**: head, file, line, col, the typed form, a one-line reason). **19 sites have no row.**
  One helper reported that `255-71-table-tests.tsv` was clobbered and restored by hand: **verify it against the census**.

## Correction (2026-09-30, builder)

An earlier draft of this amendment blamed the stop on the first agent fanning out helper agents, and added a rule against
it. **That cause was wrong:** the weekly limit was nearly spent (about 2% left) before the stone began. The rule is
withdrawn; executors operate as normal.

## Resume

1. Verify every row of `255-71-master.tsv` against `255-71-census-pre.tsv` (same site, a sensible typed form, the reason
   matches the site). Fix wrong rows. Type the 19 missing sites the same way, or list them under STOP-1's rule.
2. Commit the finished table as **data** beside this stone (EDN, as the brief says; convert the TSV), and apply it with
   `wat-scripts/fixes/typed-constructors.wat` under the stash-dance: the checker without the wall converts, then
   `git stash pop` the wall.
3. Continue the brief from there: tests, gates, SCORE. Delete `scratch/` when the table is committed (it is untracked
   scratch).

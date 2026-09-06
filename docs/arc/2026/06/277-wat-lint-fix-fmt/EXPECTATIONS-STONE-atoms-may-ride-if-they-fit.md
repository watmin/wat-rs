# EXPECTATIONS — STONE: atoms may ride if they fit

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ `{:a 3 :b 42}` | **INLINE** |
| 3 | ★ `{:a (+ 1 2) :b 42}` | one pair per line |
| 4 | ★★★ **a long all-atom map** | one pair per line — **the WIDTH path, shown firing** |
| 5 | ★★ `(:p::T :a 1)` | **INLINE** — defect 4, answered |
| 6 | ★ `(Unreadable :file p :reason (…))` | still explodes |
| 7 | ★★ the width CONTROL | derived == actual on single-line forms, synthesized nodes EXCLUDED; **CHECKED and MISMATCH both printed** |
| 8 | idempotent | `IDEMPOTENT=true` across rows 2-6 and every existing fixture |
| 9 | ★★ no rule names a column or a budget | `grep -c 'col' rules/*.wat` = 0; `grep -c '120' rules/*.wat` = 0 |
| 10 | ★★ the sibling table still wins | a table row stays a table row |
| 11 | ruled shapes hold | every existing fixture |
| 12 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0 |
| 13 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 14 | fixtures type-check | `wat --check` clean on each **before** the floor |
| 15 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 16 | ★★ the 614 doc examples | over-120 still **0**; report how many became INLINE |
| 17 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 18 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 60-90 min. The fold is copied; the emitter's decision point is the work.

## Trap-doors named in advance

- **Row 4 is the only row that proves width is consulted.** Rows 2 and 5 pass with no width computed
  at all — a strike that skips the fold entirely satisfies them. **Green on 2/5 and silent on 4 has
  not built this stone.**
- **Row 7 prints CHECKED as well as MISMATCH.** Zero mismatches over zero forms examined is
  indistinguishable from success; that vacuous green was published once in this arc.
- **Row 9's second half is new:** the budget number must appear in the emitter, ONCE. A `120` in a
  rule file is the same defect as a column in a rule file.
- **Row 10 is a conflict check, not a formality.** A table row is all-atoms and short; nothing must
  re-inline it pair-by-pair or explode it on width.
- **The vacuous green:** row 13 prints the comment COUNT.

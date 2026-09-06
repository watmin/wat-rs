# EXPECTATIONS — STONE: the sibling table

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ a 3-row keyword group | `kw-table.wat` → **3 aligned lines, not 15** |
| 3 | ★ a POSITIONAL group | same head, same arity → aligned columns |
| 4 | ★★★ **a DIFFERENT KEY ORDER is not a group** | `format.wat:16`'s shape stays ungrouped |
| 5 | ★★ non-adjacent / different heads | not grouped; untouched |
| 6 | ★★ a group of 2 IS a table | the ruling — **and report how many groups would form at 3** |
| 7 | ★★★ **the drifted real table** | `rules-corpus-02:176-180` comes out **EXACT** — uniform padding, no stray space |
| 8 | a group needing no padding | `rules-corpus-01:121-125` unchanged |
| 9 | idempotent | padding from this pass only — `IDEMPOTENT=true` |
| 10 | no rule names a column | `grep -c 'col' rules/*.wat` → **0 in every file** |
| 11 | ruled shapes hold | every existing fixture, ruled + idempotent |
| 12 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0 |
| 13 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 14 | fixtures type-check | `wat --check` clean on each **before** the floor |
| 15 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 16 | ★★ the 614 doc examples | over-120 still **0**; report how many table groups formed |
| 17 | ★★ the corpus blast radius | of the ~354 proxy candidate runs, **how many actually become tables** |
| 18 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 19 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 60-90 min. Detection is a rule; the group-scoped padding pass is the work.

## Trap-doors named in advance

- **Row 4 is the row a wrong rule passes everything else on.** Grouping on head alone satisfies rows
  2, 3, 6 and 7 and silently reorders nothing — it just destroys a call whose key order is the point.
- **Row 7 is the correction argument.** The real table has drifted; "unchanged" is a FAILURE there.
  Exactness is the pass condition.
- **Row 8 is the reverse:** a group that needs no padding must be recognised and left alone. A rule
  that only fires when padding is needed passes row 7 and fails row 8.
- **Row 6 is a REPORT as well as a gate** — the threshold is 2 by ruling, and the 3-count tells the
  builder whether to raise it on evidence.
- **Row 17 is the number nobody has:** ~354 is my proxy (3+ consecutive lines with 2+ internal
  spaces), not a census. The real count of groups is the actual blast radius.
- **The vacuous green:** row 13 prints the comment COUNT.

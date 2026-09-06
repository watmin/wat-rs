# EXPECTATIONS — STONE: the emitter survives a comment

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **the node after a comment KEEPS its indent** | `(:wat::i64::+ x 1)` at **2**, not 0 |
| 3 | ★★★ **`wat/spawn.wat`** | variants stay at indent 2 across their trailing comments, **tags still padded** |
| 4 | ★★ blank lines carry NO trailing whitespace | `grep -c ' $'` on the OUTPUT = 0 |
| 5 | ★ no blank line between ret-spec and body | |
| 6 | ★ exactly one blank line after each top-level form | |
| 7 | ★★★ **a blank line INSIDE a body is PRESERVED** | rule 5 is ONE position, not a purge — 869 such lines exist |
| 8 | ★★ idempotent | the required blank does not accumulate on pass 2 |
| 9 | ★★★ **no comment is LOST** | `wat/io.wat` **28** · `wat/deporder.wat` **85** — both printed |
| 10 | ★★ REPORTED, not fixed | what became of each TRAILING comment — defect E, the builder's call |
| 11 | ruled shapes hold | defenum · defrecord · defn `<-` · table · atom-map · kwargs |
| 12 | the 614 doc examples | over-120 still **0** |
| 13 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0; `col`/`120` 0 |
| 14 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 15 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 16 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 45-75 min. The column state is the work; the blank-line rules are small.

## Trap-doors named in advance

- **Row 3 is a REAL FILE and is the builder's own example.** Every fixture in this arc is
  comment-free, which is precisely why this defect survived nine stones. **A fixture cannot prove
  row 3.**
- **Row 7 is what stops rule 5 becoming a massacre.** 869 blank lines sit inside top-level forms; a
  strike that reads "no blank lines in a form" passes 5 and 6 and destroys all of them.
- **Row 9 is a LOSS check, not a preservation claim.** Both counts are printed; a drop is a red.
- **Row 10 is a REPORT.** Defect E is unruled and choosing would be worse than leaving it.
- **`\;` is a char literal.** A pass that scans text rather than using the lexer's spans re-breaks it.

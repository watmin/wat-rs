# EXPECTATIONS — STONE: the definition forms and the aligned `<-`

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ `defrecord` | NAME rides the head line; fields one per line; **`<-` ALIGNED** |
| 3 | ★ `defstruct` / `defenum` | the same |
| 4 | ★★★ **`defn`'s arg-spec `<-` ALIGNED** | uneven names (`self` / `work-fn` / `c`) → arrows in ONE column |
| 5 | ★★ a `defenum` with MIXED variants | `wat/spawn.wat:195`'s shape — bare + field-carrying — **not broken** |
| 6 | idempotent | `IDEMPOTENT=true` on rows 2-5 and every existing fixture |
| 7 | ★★ no rule names a column | `grep -c 'col' rules/*.wat` = **0**; `grep -c '120'` = **0** |
| 8 | prior rules still win | sibling table · all-atoms inline · kwarg alignment · ret-spec one line |
| 9 | ruled shapes hold | every existing fixture |
| 10 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0 |
| 11 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 12 | fixtures type-check | `wat --check` clean on each **before** the floor |
| 13 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 14 | ★★ the 614 doc examples | over-120 still **0**; **the two `defrecord`s in the step-payload example have their names on the head line** |
| 15 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 16 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 60-90 min. Three rule pairs are mechanical; the stride on the alignment pass
is the work.

## Trap-doors named in advance

- **Row 4 is old debt and the point of the stone.** Rows 2-3 are satisfiable without touching `defn`
  at all. A strike green on 2-3 and silent on 4 has done the easy half.
- **Row 4 needs UNEVEN NAMES to prove anything.** `[x <- i64 y <- i64]` looks aligned whether or not
  the rule fires — the fixture must use names of different lengths.
- **Row 5 is where a naive rule dies.** `defenum`'s bare and field-carrying variants share one child
  list; a fixed-position assumption breaks every enum in the corpus.
- **Row 14 is the real-input check**, and it is specific: the two `defrecord`s inside the
  step-payload example currently render with the name dropped to its own line.
- **The vacuous green:** row 11 prints the comment COUNT.

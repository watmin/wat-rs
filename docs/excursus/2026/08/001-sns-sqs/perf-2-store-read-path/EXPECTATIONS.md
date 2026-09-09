# EXPECTATIONS — perf 2: the store's read path

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **cost is flat in table size** | `probe-store-scan-cost.wat` | roughly flat across 250/500/1000 rows. Baseline **1691 / 3489 / 9204 ms**. Still climbing ~2× = the walk survives |
| 2 | ★ **the differentials hold** | the five mem-vs-sqlite tests | all pass, **unedited**. Any red = behaviour moved (STOP-2) |
| 3 | the circuit is faster and still correct | `wat-scripts/fanout/circuit.wat` | `n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;…` and materially under **287 s**. Report the number |
| 4 | the durable Record is unchanged | `git diff` on the Record | shape identical (STOP-1) — no wire or hibernation change |
| 5 | hibernate/resume rebuilds | a resumed store | reads answer identically |
| 6 | `scan-index` got faster too | the index path | it is the queue's hot path; a fix that only helps `scan` misses the circuit |
| 7 | put-is-a-replace holds | the excursus 001 stone 2c behaviour | unchanged |
| 8 | sqlite untouched | `git diff wat/query/sqlite-store.wat` | empty (STOP-4) |
| 9 | the header is updated | `wat/query/mem.wat` head | describes what it now does — a stale "correct, not fast" is FM 22 |
| 10 | no runtime/surface change | `git diff --stat src/`; Store op count | empty; unchanged |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5163+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 90–150 minutes. The index maintenance across put/delete/replace carries it,
not the read.

## Trap doors, named in advance

- **Putting the index in `:durable`.** Fastest thing to write; changes the wire format and the
  hibernation format for a perf fix. Row 4.
- **Fixing `scan` and not `scan-index`.** Passes row 1 and fails the circuit. Row 6 exists for this.
- **Adjusting a differential** to make it pass. Row 2 says unedited; a moved differential is the
  oracle being silenced.
- **A correctness "improvement" riding along.** If the old code was wrong somewhere, that is a
  separate finding. Row 7 and STOP-3.
- **Firing on nothing:** rows 2–11 all pass if the code is merely tidied and still walks the table.
  Row 1 is the only one that catches it.

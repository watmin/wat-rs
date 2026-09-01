# EXPECTATIONS — perf 3: the indexed vector update

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **write cost stops growing** | `probe-store-write-cost.wat` | per-doubling approaches **2×**. Baseline puts 400/1333/4887 ms (3.33×, 3.67×), deletes 751/2801 ms |
| 2 | ★ **the differentials hold** | the five mem-vs-sqlite tests | all pass, **unedited**. Swap-remove is the likeliest thing to trip them (STOP-1) |
| 3 | the circuit, **measured not predicted** | re-run `circuit.wat` | same output string; wall time **reported** against 257 s. No target — perf-2's row 3 predicted one from the wrong measurement |
| 4 | order-independence verified, not assumed | the sites touching `Record/rows` | confirmed by the strike itself (STOP-2) |
| 5 | out-of-range is loud | `set` past the end; `drop-last` on empty | located errors naming index and length (STOP-4) |
| 6 | the primitive is narrow | the new core surface | `set` + `drop-last` only (STOP-3) |
| 7 | durable shape unchanged | `git diff` on the Record | identical — perf-2's contract decision still holds |
| 8 | sqlite untouched | `git diff wat/query/sqlite-store.wat` | empty |
| 9 | hibernate/resume | a resumed store | reads answer identically |
| 10 | header updated | `wat/query/mem.wat` head | describes the write path as it now is |
| 11 | reads did not regress | `probe-store-scan-cost.wat` | still flat, ~119/116/123 ms |
| 12 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 0 failed, FLOOR=0 |

**Runtime prediction:** 90–150 minutes. The primitive is small; index-position maintenance under
swap-remove is the real work.

## Trap doors, named in advance

- **Swap-remove without fixing the moved row's index position.** Silently corrupts lookups for one
  row per delete. The differentials are the only thing that catches it — row 2.
- **A silent no-op on a bad index.** The swallow this arc removed everywhere else. Row 5.
- **Predicting the circuit's number.** Row 3 says report, not promise. That mistake is one stone old.
- **Firing on nothing:** rows 2–12 all pass if the primitive lands and the store still folds.
  Row 1 is the only one that catches it.

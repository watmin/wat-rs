# EXPECTATIONS — the instrument reports what happened

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ This stone must change **no** measured quantity except by relabelling. Its gates are about
things NOT moving.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ⛔⛔ **publish and drain do not move** | `circuit.wat` ×3, shipped config | `publish` ~20800 ±300, `drain` ~190. **A move means something other than a timer changed** |
| 2 | ★★ **`stop` becomes teardown only** | the phases line | `stop` ~1800, down from ~6800 |
| 3 | ★★ **`collect` appears and carries the rest** | the phases line | `collect` ~4800 |
| 4 | ⛔ **the phases sum to the wall** | `setup + publish + drain + collect + stop` vs total | within a few ms. An unaccounted remainder is a finding |
| 5 | ★★ **`publish-attempts` counts partial resends** | compare to `full-retries` | `publish-attempts` > `full-retries + publish-calls`. If they are equal, the partial branch is still not counted |
| 6 | ⛔ **`full-retries` keeps its meaning** | compare to today | ~520–600 at m=4, unchanged. It still counts bounces only |
| 7 | ⛔ **delivery exact** | ×3 | `total=8000; distinct=8000; dup=0` |
| 8 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 9 | ⛔ **blast radius** | `git status --porcelain` | `circuit.wat` + SCORE. **Nothing else** |

## REPORTS

| ▪ | what |
|---|---|
| a | the full phases line, before and after |
| b | `publish-attempts` — the number that makes a per-call figure computable |
| c | `collect`'s internal split, if cheap to keep: `sum-disrupts` was 1262 ms and `collect-stop` 3509 ms |

## RUNTIME

20–35 min. Moving a timestamp and threading a counter.

## TRAP DOORS

- ⚠⚠ **Row 1 is the whole stone.** If `publish` moves, a timer was not the only thing that
  changed, and every comparison in this arc rests on `publish` being stable across harness edits.
- ⚠ **`empty-flags` does a `receive` per queue.** Decide deliberately whether it is collect or
  teardown and say which — it touches the system, unlike the pure stats calls.
- ⚠ **Do not pin the delay for this measurement.** `asleep` sums the drawn value; pinning makes it
  lie, which is recorded in the DESIGN because it produced a bad number for me today.
- ⚠ **`publish-attempts` must count the PARTIAL branch.** That branch currently passes `retries`
  through untouched, which is precisely the invisibility being fixed; a counter added only to the
  `Accepted 0` path reproduces the bug with a new name.

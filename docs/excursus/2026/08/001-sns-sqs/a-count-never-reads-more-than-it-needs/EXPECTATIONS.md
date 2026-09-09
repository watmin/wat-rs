# EXPECTATIONS — a count never reads more than it needs

Written before the strike. Graded by my own re-run of every row.

⛔ = GATE. ▪ = REPORT.

★ **No timing is gated and none is expected to move.** The DESIGN predicts neutral at `cap 64`.
This stone is judged on the count being bounded, the two impls agreeing, and nothing breaking.

## GATES

| # | what must hold | how I check it | expected |
|---|---|---|---|
| 1 | ★★ **the count saturates** | probe: 5000 rows, `limit 65` | `Ok 65` — **not 5000** |
| 2 | ★★ **and is exact below the limit** | same index, `limit 100000` | `Ok 5000`. A count that always returns the limit is not a count |
| 3 | ★★ **both impls agree** | same request against `mem-store` and `sqlite-store` | identical `n` at several limits. `mem` is the oracle; a divergence here is worse than the cost being fixed |
| 4 | ⛔ **the bound is actually passed** | `grep "_lim\|_now-ns" wat-scripts/queue/sqs.wat` | the `total`/`depth` closures no longer discard them |
| 5 | ⛔ **the cap gate still gates** | probe: fill to cap, send one more | `Accepted 0`. Off-by-one here silently changes admission |
| 6 | ⛔ **delivery exact** | `circuit.wat` ×3 | `total=8000; distinct=8000; dup=0` |
| 7 | ⛔ **publish does not move** | ×3 | ~20700 ±300. A *gain* here would be as surprising as a loss and should be explained, not celebrated |
| 8 | ⛔ **the floor** | `./scripts/floor.sh`, **Summary line** | `0 failed`. ▪ count reported |
| 9 | ⛔ **blast radius** | `git status --porcelain` | the four briefed files + probes + SCORE. **No `src/`, no `service.wat`, no `circuit.wat`** |

## REPORTS

| ▪ | what |
|---|---|
| a | rows examined per count, before and after, if the impl can report it |
| b | `publish` / `collect` / `stop` medians — expected flat |
| c | whether any caller genuinely wanted an unbounded count (STOP-3's answer, even if it did not fire) |

## RUNTIME

30–50 min. One field, two impls, two callers.

## TRAP DOORS

- ⚠⚠ **`cap + 1`, not `cap`.** The gate needs to distinguish "exactly at cap" from "over cap". A
  limit of `cap` saturates at `cap` in both cases and the gate silently stops rejecting. Row 5.
- ⚠⚠ **The mem impl is the oracle.** Other tests compare sqlite against it. A saturating sqlite
  and an exhaustive mem would pass rows 1 and 2 separately and disagree in row 3 — and that
  divergence is a worse defect than the unbounded read.
- ⚠ **A gain on `publish` needs explaining, not accepting.** The DESIGN predicts neutral at this
  depth. If it speeds up, the model of where the time goes is wrong again and that matters more
  than the milliseconds.
- ⚠ **This unblocks the deep-drain benchmark; it is not that benchmark.** Do not raise a `:cap` or
  add a depth test here.

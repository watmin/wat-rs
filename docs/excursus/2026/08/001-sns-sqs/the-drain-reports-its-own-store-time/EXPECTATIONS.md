# EXPECTATIONS — the drain reports its own store time

Written **before** the strike. Judged on **the instrument**, never on what it shows.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ both fields reported | `… circuit.wat 500 4 3 8192 true` | `drain-store-ms=` and `drain-store-calls=` on the phases line |
| 2 | the delta is per queue | read the format expression | **divided by `m`** — not the raw sum |
| 3 | it is a drain measurement | any run | `drain-store-ms` **≪ `store-ms`** (whole-run), and plausibly ≤ `drain` |
| 4 | it is bounded and sane | n=500 / 1000 / 2000 | ≥ 0, monotonic in n |
| 5 | consistency with the poller | compare `drain-store-calls × m` to `(poll-calls/(m+1)) × m × 2` | the poller's share is a **fraction** of it, not larger |
| 6 | no `sqs.wat` change | `git diff --stat` | **`circuit.wat` only** |
| 7 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; both fields additive |
| 8 | correctness untouched | n=2000 fill-first | `total=8000;distinct=8000;dup=0` |
| 9 | overhead invisible | n=500 / 1000 / 2000 | `pairs/sec` within **±15 %** of 3767 / 3130 / 2423 |
| 10 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 11 | scripts load | `every_wat_scripts_file_loads` | green |
| 12 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **No row asserts anything about `drain-store-ms / drain`.** That ratio is the entire output.

⚠ **Row 5 is a sanity check, not a gate on the answer.** The poller contributes ~6–10 % of all store
calls (`sqs.wat:976` — every `stats` costs 2). If the drain delta came out *smaller* than the
poller's own derived contribution, the sampling is wrong.

⚠ **Nothing is banded that varies run to run.** `store-calls` at n=1000, same code, zero retries:
4959 / 4982 / 4996 / 5002. Four rows in this arc have policed noise; this one will not.

## Runtime prediction

**20–30 minutes.** Four calls to two existing functions, two subtractions, two format fields, one
file. The floor is the long pole.

## Trap-doors named in advance

- **Sample and timestamp must bracket the same span.** If the sample is taken after `t-drain0`, its
  own 8 store calls are inside the delta; if before, they are not. Either is fine — **say which.**
- **`/ m`, not raw.** The sums are across m queues.
- **Nanos to millis at the format**, as `store-ms` already does.
- **`drain-store-calls` is a count, `drain-store-ms` a duration** — do not divide one by the other
  and call it a per-call cost without stating the phase mix, which is how the last stone's number
  got over-read.

## The fork this resolves — stated so it cannot be retrofitted

- **`drain-store-ms`/pair grows in step with `drain`/pair (+61 %)** → the drain's time is inside the
  store; the next question is the SQL, in the **stdlib**.
- **flat or far below** → **the store is exonerated for the drain**; the time is in the queue's own
  processing or in scheduling, and the search moves somewhere nobody has looked.

★ **No prediction.** And unlike the previous stone, this instrument **can** discriminate: it
measures the phase the fork is about.

## What this stone does NOT claim

⚠ **It does not explain the slope.** It measures the drain instead of the whole run.

⚠ **It does not separate the store's own work from the IPC.** `store-ns` is the round trip as the
queue sees it; that split lives in the stdlib and is only worth doing if this points there.

⚠ **It does not remove the observation cost.** Every `Queue/stats` will still cost 2 store calls;
the arc has already ruled against caching depth in service state
(`docs/excursus/2026/08/001-sns-sqs/stop-fetching-rows-to-get-a-number/SCORE.md`). Naming it is this stone; changing it is not.

# EXPECTATIONS — `store-ns`

Written **before** the strike. Judged on **the instrument**, never on what it shows.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ `store-ms` is reported | `… circuit.wat 500 4 3 8192 true` | `store-ms=` on the phases line |
| 2 | `ns` is a named field on `TakeAcc` | `grep -n 'defstruct :queue::TakeAcc' -A 5 sqs.wat` | four fields; **no new Tuple nesting anywhere** |
| 3 | nested-Tuple constructions still zero | `awk '{c=gsub(/:wat::core::Tuple /,""); if(c>=2) t++} END{print t+0}' sqs.wat` | **0** — the last stone's gain is not given back |
| 4 | every counting site also times | read the eight `Store/*` sites | **8 of 8**; none counts without timing |
| 5 | it is bounded and sane | any run | `store-ms` ≥ 0, and **`store-ms` < `total`** |
| 6 | it tracks the work | n=500 / 1000 / 2000 | grows monotonically with n |
| 7 | overhead is invisible | n=500 / 1000 / 2000 | `pairs/sec` within **±15 %** of 3623 / 3050 / 2303 |
| 8 | no-args unchanged | `… circuit.wat` | every **pre-existing** field identical; `store-ms` additive |
| 9 | correctness untouched | n=2000 fill-first | `total=8000;distinct=8000;dup=0` |
| 10 | the eight ripple sites took `_` only | `git diff -- wat-scripts/topic wat-scripts/scratch-pad` | **8 insertions, 8 deletions**, every one a trailing `_` |
| 11 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 12 | scripts load | `every_wat_scripts_file_loads` | green |
| 13 | the floor | `scripts/floor.sh` | **read the Summary line**: 5221 passed, 22 skipped |

⚠ **`store-ms` may legitimately EXCEED `drain`.** It is cumulative over the whole run and the fill's
puts are in it. That is expected, not a bug — report both.

⚠ **No row asserts anything about `store-ms / drain`.** That ratio is the entire output and must not
appear as an expectation.

⚠ **`store-calls`/pair is deliberately NOT banded.** Measured this session at n=1000, same code,
zero retries: **4959 / 4982 / 4996 / 5002** — a ±0.005/pair spread. Report it; a band would police
noise. Four rows in this arc have already made that mistake.

⚠ **Row 7 is the one that could quietly fail.** The overhead is priced at 0.20 %, but priced is not
measured *in situ*. Beyond the band means the instrument is changing what it measures — a finding,
not a rounding error.

## Runtime prediction

**40–60 minutes.** One named field, one `:ephemeral` field, eight clock pairs at sites already being
edited, one sum, one format, eight known ripple sites. The floor is the long pole.

## Trap-doors named in advance

- **Nanos on the wire, millis at the format.** A single store op is ~100–400 µs; per-op millisecond
  truncation would floor most of them to 0.
- **The clock pair must bracket only the call.** Including the surrounding `State` rebuild folds
  queue-side work into `store-ns` and **pre-decides the fork this stone exists to resolve**.
- **`TakeAcc/ns` inside the folds, `store-ns` outside.** The folds accumulate into the struct; the
  arm adds it on afterwards, exactly as `calls` → `store-calls` already works.
- **Append `store-ns` LAST** on `StatsResponse::Ok`, after `store-calls`. A middle insertion
  silently reassigns every binding after it across 15 sites.
- **Copying `store-calls` into `store-ns` by cut-and-paste is not a compile error** and would
  produce a plausible number that is really a count.

## What this stone does NOT claim

⚠ **It does not explain the slope** (3623 / 3050 / 2303 pairs/sec). It halves the search space — the
third halving, after store-contention was eliminated structurally and the count explanation was
measured dead.

⚠ **It does not time the store's internals.** `store-ns` is the round trip as the queue sees it:
the store's own work **plus** the IPC. Separating those is the next question, and it lives in the
stdlib.

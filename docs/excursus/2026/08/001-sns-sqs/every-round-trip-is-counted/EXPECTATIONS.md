# EXPECTATIONS — every round-trip is counted

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the instrument adds no round-trips** | read the diff | every count is either the caller's own local tally or rides a reply already being made. **No new `send`/`recv` anywhere** |
| 2 | ⛔ **the phase timings do not move** | 3 runs before, 3 after | `setup`/`fill`/`arm`/`drain`/`collect`/`stop` within band. A material shift means the instrument costs something — STOP-2 |
| 3 | ⛔ **correctness untouched** | `2000 4 3` ×3 | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise |
| 4 | ★ **the budget is reported by peer class** | the phase line | `rt-store`, `rt-queue`, `rt-seen`, `rt-worker`, `rt-topic`, `rt-total` — or an explicit `unknown` for any class that cannot be counted free |
| 5 | ★ **the seen store is no longer invisible** | the budget | a non-zero `rt-seen`. It is called per delivery and has never appeared in any number |
| 6 | ★ **anything uncountable is named, not omitted** | the SCORE | each `unknown` says which class and why a free count was impossible |
| 7 | **the stall gate still fires** | `circuit.wat 5 1 0 32 false 0` | rc 2, `drained-stalled` |
| 8 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 9 | **the floor holds** | `scripts/floor.sh` | **Summary line**: 5237 passed, 22 skipped, 0 FAIL, 0 TIMEOUT |
| 10 | **blast radius** | `git status --porcelain` | `circuit.wat` + the SCORE. `sqs.wat` or `wat/` is STOP-3 |

## ★★ Row 11 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 11 | ★★ **`rt-total` ≥ 13 800 at `2000 4 3`** | my arithmetic, falsifiable |

⛔ I reported **5804** round-trips and called it a floor. Then I found the seen store alone is ≥8000
uncounted, giving **≥13 800**. **That is still arithmetic from counters I have not built**, and it is my
third round-trip estimate today — the first was 55× wrong (`sum-disrupts`), the second more than 2× short.

**If the measured total is materially below 13 800, my estimate was wrong again. If it is materially
above, the invisible traffic is worse than I said.** Either way report the number and the split; do not
reconcile it to my figure.

## ⚠ Row 12 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 12 | ⚠ no speedup, and no network cost model | stated in the SCORE |

**This makes nothing faster.** A SCORE reporting a speedup has measured the wrong thing.

And a count is **not** a cost model. If the SCORE prices the budget at an RTT, it must say that the
serialised arithmetic **overstates** by whatever concurrency is achieved — measured at roughly **3×**
across ~24 processes (16.3 s serialised against a 5.4 s real messaging path). ⛔ Do not print a
cross-region figure without that caveat beside it.

## Runtime prediction

**90 minutes–2 hours.** The counters are small; the risk is entirely in row 1 — finding a free channel for
the consumer's and worker's internal counts. The `:stop` projection is the intended one.

## Trap-doors

- ⛔ **The obvious way to count a callee's calls is to ask it. That is the defect.** `sum-disrupts` already
  costs a round-trip per worker and waits out the worker's 250 ms poll. Adding a `Seen/stats`-style call
  per consumer would repeat it. **Ride the stop projection.**
- **`:stop` projections are author-chosen** (`wat/service.wat:2928`) and already carry final state — a
  worker's own tallies belong there.
- **`store-calls` is already per tier** and reconciles to `put+delete+count+scan` with remainder 0. Do not
  break that identity; `rt-store` should be derivable from it, not a parallel count that can disagree.
- **`poll-calls` is the harness's own poll loop** and is ~95 % of stats traffic. It belongs in the budget
  as its own line, not folded into `rt-queue`, or the biggest item disappears into a subtotal.
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, the tick, or the poll
  wait.** Before/after comparability depends on the topology being byte-identical.

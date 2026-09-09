# EXPECTATIONS — the inbox visibility stops being an outlier

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **`dup = 0` at EVERY swept value** | every cell of the sweep | **0**, always. One non-zero ends the sweep — STOP-1 |
| 2 | ⛔ **the happy path is intact** | `2000 4 3 8192 true 1000` ×3 at the chosen default | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise |
| 3 | ⛔ **the stall gate still fires** | `circuit.wat 5 1 0 32 false 0` (j=0, no consumers) | rc 2 with `drained-stalled`. **A visibility change must not make a genuine stall undetectable** |
| 4 | ★ **the sweep is complete and reports all four columns** | the SCORE's table | `inbox-vis-ms` ∈ {5000,1000,500,200,100} × {idle ×3, 8-way}, each with `dup`, inbox `redeliveries`, `drain`, slow-mode yes/no |
| 5 | ★ **the 67× term shrinks** | the sweep's `drain` column | at some value the ~5.2 s slow mode is materially reduced or gone. ⚠ If it is not, say so — the constant was not the cause |
| 6 | ★ **the cost is quantified, not asserted** | inbox `redeliveries` across values | a number per value, with the idle/loaded split. *"No extra redeliveries"* is a claim needing a column |
| 7 | **`sns-fanout.wat` is untouched** | `git diff` | `mk-tw` already takes the value; the topic file should not move |
| 8 | **the five passing chaos scenarios still pass** | `--run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'` | `7 tests run: 7 passed` — the state `18a86fe27` reached |
| 9 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 10 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |
| 11 | **blast radius** | `git status --porcelain` | `circuit.wat` + the SCORE. Anything forced, named |

## ★★ Row 12 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 12 | ★★ **the outlier is wrong** | a shorter visibility is a net win |

⛔ **This is my recommendation, not a fact.** I am asserting that 5 s at `circuit.wat:2323` is an
accident and 200 ms — the value at six other sites — is closer to right. **If the sweep shows the
opposite** — that short visibilities cost more in premature redelivery and duplicate fan-out than they
buy in latency — **then the outlier was correct, my reading of "one site disagreeing with six" was a
pattern-match rather than an argument, and the stone lands as a documenting comment instead of an
edit.** Report that outcome; it is complete and valuable.

## ⚠ Row 13 — what this stone must NOT be credited with

| # | what | expected |
|---|---|---|
| 13 | ⚠ **it does not fix the chaos reds** | those were fixed at `18a86fe27` |

The two chaos reds were resolved by progress-bounding the poller, without touching any rate, cap, or
timeout. **They must remain fixed by that mechanism.** If any SCORE sentence implies this stone fixed
them, or that the knob was needed to make something pass, it is wrong.

⚠ It also does not touch the residual drain slope, `scan-index`, `count-index`, `TakeAcc`, the
poller's own round-trips, or the inbox's absent *fault* injection.

## Runtime prediction

**90–120 minutes.** The edit is small; the sweep is 5 values × (3 idle + 1 eight-way burst) ≈ 20
invocations of a ~10 s scenario, plus the write-path confirmation and the floor (~8 min).

## Trap-doors

- ⚠ **Visibility shorter than the receive's own wait.** `sns-fanout.wat:417` claims with
  `:wait (UpTo (Milliseconds 250))`. At 200 ms and below, an entry can expire while the worker that
  claimed it is still working — that is exactly what row 6 measures. Six precedent sites already do
  this, so it is not novel, merely unmeasured at this scale.
- **The subscriber `vis` and the inbox visibility are DIFFERENT knobs.** `vis-ms` (`circuit.wat:2255`)
  feeds the sub queues at `:471`/`:943`. Do not merge them; the whole reason the ~5.2 s gap went
  unexplained for hours is that varying one looked like varying the other.
- **`vis-ms = 0` means 200 ms *only when a drop rate is set***, else 1000 s. Whatever default you pick
  for the inbox, do **not** inherit that 1000 s branch — an inbox entry that never redelivers turns a
  lost ack into a permanent stall.
- **`run-with` reaches 16 parameters.** Add at the end, keep it optional, and do not reorder — every
  existing caller and fixture positions by index.
- **Do not change `:cap`, `sub-cap`, `:max-entries`, or `vis-ms`.** The sweep's comparability depends
  on one variable moving.
- **The box must be quiet before every timing cell**, and the 8-way bursts go through `capped.sh`.

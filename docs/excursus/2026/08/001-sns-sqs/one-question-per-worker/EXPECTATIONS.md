# EXPECTATIONS — one question per worker

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **correctness untouched** | `2000 4 3` ×3 | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise |
| 2 | ⛔ **every summary field survives** | the phase line | `disrupts`, `check-exhausted`, `mark-exhausted`, `ack-retries`, `ack-exhausted` all still printed, same values |
| 3 | ⛔ **one question per worker** | read the diff | `sum-disrupts` gone; exactly one fold over the workers remains |
| 4 | ★ **`rt-worker` falls by 12** | the budget line | 36 → 24 at m=4 j=3. It is 3 folds × 12 workers today; one fold goes |
| 5 | ★ **`collect` falls materially** | interleaved, ≥3 pairs | direction with separated bands. ⚠ I name no figure — see row 8 |
| 6 | **the stall gates still fire** | `5 1 0 32 false 0` and `50 2 2 32 true 0` | rc 2 `drained-stalled`; rc 2 `filled-stalled` |
| 7 | **chaos, corpus, floor** | the three | `7 passed`; PASS; **Summary line** 5237 passed, 22 skipped, 0 FAIL |

## ★★ Row 8 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 8 | ★★ **removing one of two questions removes about half the per-worker term** | direction, with bands |

⛔ **This is my attribution, not a fact.** I claim `collect`'s per-worker cost is ~338 ms/worker at the
shipped 250 ms poll because **two** questions each wait out ~half a poll. If removing one question does
**not** roughly halve it, the model is wrong.

⚠ **I name no percentage on purpose.** My record this session is **eight** quantitative claims refuted —
55× high, 2× low, 19 % high, a 19× re-ranking error, an exactness claim killed by a nullary variant I had
listed as a candidate, and a `fill-excess = m × redeliveries` law I refuted myself two minutes after
writing it. **Direction with separated bands is all I will accept from myself here.**

## ⚠ Row 9 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 9 | ⚠ no throughput win; no "worker is now answerable" | stated in the SCORE |

⚠ **This does not make a parked worker answerable.** It reduces how often we ask, not what asking costs.
The ~150 ms per question stands. The real fix is the `asks` gap named at `a4f2d7f7b`, which reaches `src/`
and is the builder's to open.
⚠ And `collect` is ~76 % process-lifecycle and observability. **Halving one term of it does not make the
system faster** — the drain and `distinct`/`dup` are untouched.

## Runtime prediction

**90 min–2 hours.** The projection change is small; the risk is row 2 (every field surviving) and the
`Tuple`-has-no-fourth-accessor trap.

## Trap-doors

- ⛔ **`Tuple` has no fourth accessor** (`wat/core.wat:1737`) — hit **twice** in this campaign. More than
  three values out means a **record**, not a wider tuple.
- ⛔ **Never `println` from a service handler.** It corrupts the frame stream in a forked service —
  measured today as `defservice stop: expected Status::Stopped`, and documented nowhere.
- **`collect-stop` currently conj's `Outcome`s through a nested fold.** Widening what stop returns changes
  that fold's element type; the compiler will name every site.
- **`disrupt-points` is a `String` and normally `""`.** It is returned and discarded today; do not assume
  it is unused by something else without checking.
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, the tick, or the poll
  wait.** Comparability depends on the topology being byte-identical.

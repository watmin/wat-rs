# EXPECTATIONS — the fill poller gets what the drain poller got

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **THE CHECK CAN STILL GO RED** | build a system that stops filling; run it | a non-empty verdict, rc 2. **A poller that cannot fail is worse than the one it replaces.** Say how you convinced yourself |
| 2 | ⛔ **the unsatisfiable test is gone** | read `sweep-filled?` | `>=` on the visible term. An overshoot can no longer make completion unreachable |
| 3 | ⛔ **the overshoot is REPORTED, not swallowed** | a run, and the diff | when completion passes with `visible > n`, the excess appears in the output. `>=` must not bury the 2010 fact |
| 4 | ⛔ **the happy path is unchanged** | `2000 4 3` ×3 interleaved | `distinct=8000`, `dup=0`, `fill` within band. A successful fill never reaches the give-up path |
| 5 | ★ **progress-bounded, not attempt-bounded** | read the diff | no `left`/attempt budget governing the give-up; `Σ(visible+unacked+acks)` and a wall ceiling do |
| 6 | ★ **three verdicts, distinguishable** | the verdict strings | `filled-stalled` and `filled-timeout` distinct, each naming the signal, `stale`, `stale-max`, `elapsed`; `filled-unread` still outranks |
| 7 | ★ **no new crossings per poll** | read the diff | one sweep per iteration, as today. `rts'` accounting unchanged |
| 8 | ★ **the sibling audit is reported** | the SCORE | `sweep-unread?`, `snapshot-str`, `sweep-drained?`, the `:2650` budget — each stated, **including where it found nothing** |
| 9 | **the stall gate on the drain side still fires** | `circuit.wat 5 1 0 32 false 0` | rc 2, `drained-stalled`. This stone must not disturb the sibling it is copying |
| 10 | **chaos still passes** | `--run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'` | `7 tests run: 7 passed` |
| 11 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 12 | **the floor holds** | `scripts/floor.sh` | **Summary line**: 5237 passed, 22 skipped, 0 FAIL, 0 TIMEOUT |
| 13 | **blast radius** | `git status --porcelain` | `circuit.wat` + the SCORE. Anything forced, named |

## ★★ Row 14 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 14 | ★★ **`Σ(visible + unacked + acks)` is monotone** | verified from the code |

⛔ **My whole signal choice rests on this**, and I chose it *after* rejecting `Σvisible` for exactly this
reason (consumers drain during fill when `fill-first? = false`). **If a redelivery re-increments `visible`
without incrementing anything else, or `acks` can reset, or a claim moves a row between terms in a way
that double-counts — the signal is not monotone and my design is wrong.** Verify from the code.

⚠ My record today: **five estimates, all wrong** — 55× high, 2× low, 19 % high, a 19× re-ranking error,
and an exactness claim refuted by a nullary enum variant I had listed as a candidate and asserted away.
**Row 14 is where that pattern lands next.**

## ⚠ Row 15 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 15 | ⚠ no speedup, and the overshoot is not explained | stated in the SCORE |

⚠ **The win is entirely in the failure path** — 356 s and 40 000 crossings become bounded — and that path
does not appear in a green run's numbers. **A reported speedup means the completion path changed, which
is a finding, not a success.**

⚠ And **this stone does not explain the overshoot.** `>=` makes 2010 a pass; the mechanism is still
unknown. Any sentence implying the 2010 case is *resolved* is wrong. ★ The unclaimed coincidence stays
unclaimed: the ack-drift law is `10 × ack-retries` and the overshoot is exactly 10.

## Runtime prediction

**2–3 hours.** The loop rewrite is mechanical against a worked exemplar; the risk is rows 1 and 14 —
witnessing a genuine fill stall, and proving the signal monotone.

## Trap-doors

- ⛔ **`Σvisible` is the obvious signal and it is wrong.** With `fill-first? = false`, consumers are armed
  before the fill and `visible` falls. Use the three-term sum.
- **`filled-unread` must stay and must outrank** — a stats reply that could not be read is a different
  failure from either new verdict.
- **The drain poller's constants are 600 polls and `30000 + 12×pairs` ms.** Mirror the *shape*; the fill
  phase has a different duration profile, so state your values' reasoning rather than copying the numbers
  blind. ★ The drain stone measured `drain-stale-max=197` **on a passing run** against a first guess of
  200 — three polls from a false red. **Measure your own stale-max before choosing K.**
- **`(* n m)` at `:2650`** is the old budget. Removing it changes the call's arity — the compiler will
  name every site.
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, `:max-entries`, the tick, or the poll
  wait.** Comparability depends on the topology being byte-identical.

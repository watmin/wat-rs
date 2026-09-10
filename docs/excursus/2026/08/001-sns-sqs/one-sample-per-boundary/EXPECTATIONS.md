# EXPECTATIONS — one sample per boundary

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the identities hold exactly** | every tier of every run | `put+delete+count+scan` calls and ns sum to `store-calls`/`store-ns`, **remainder 0** |
| 2 | ⛔ **the run is still correct** | `2000 4 3 8192 true 1000` ×3 | `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise |
| 3 | ⛔ **the stall gate still fires** | `circuit.wat 5 1 0 32 false 0` | rc 2, `drained-stalled`. A sampling change must not blind it |
| 4 | ★ **the round-trips are actually gone** | `grep -c 'Queue/stats'` per fold, and the call-site count | the five one-field folds no longer exist as separate per-queue round-trips; **11 → the number of boundaries** |
| 5 | ★ **every phase has a busy figure** | the phase line | `fill`, `drain`, `collect`, `stop` each report busy-ms. `drain-busy-ms` keeps its name and its arithmetic |
| 6 | ★ **no sample sits inside a phase** | read the diff | samples are taken only at boundaries. A mid-phase sample inflates the phase it measures |
| 7 | **the five chaos scenarios still pass** | `--run-ignored ignored-only -E 'test(probe_arc278_sane_circuit)'` | `7 tests run: 7 passed` |
| 8 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |
| 10 | **blast radius** | `git status --porcelain` | `circuit.wat` + the SCORE. Anything forced, named |

## ★★ Row 11 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 11 | ★★ **`collect` drops materially** | its 4583 ms is round-trip latency |

⛔ **This is my hypothesis and the stone rests on it.** I claim `collect` costs 4583 ms because it makes
~8 stats round-trips per queue back to back. **If removing them barely moves it, I am wrong about where
`collect`'s time goes** — and that is the finding. Report the numbers and STOP rather than hunting a
second change to make the row pass.

⚠ **I am deliberately naming no percentage.** Today I twice predicted a magnitude from arithmetic I had
not done, and both were refuted. The row asks for **direction with separated bands** on 3 runs before
and 3 after.

## ⚠ Row 12 — what must NOT be claimed

| # | what | expected |
|---|---|---|
| 12 | ⚠ **no throughput claim; no interpreter claim** | stated plainly in the SCORE |

This removes **harness** round-trips. It does not make the system faster per message — `drain` may
barely move. And the new busy metrics are **server-side handler time**, which bounds how much of a phase
is *waiting* and does **not** prove the remainder is interpretation. The objective is to reach the point
where interpretation is the lag; this stone moves toward it without measuring it.

## ⚠ Row 13 — absolute figures may shift, and each shift must be named

| # | what | expected |
|---|---|---|
| 13 | ⚠ **any moved value is explained** | direction + reason, per figure |

Eleven reads at eleven instants become one read at one instant, so `store-calls`, `store-ns`,
`receive-calls`, `ticks` and the deltas derived from them may differ from today's. That is an
**improvement in consistency**, not a regression — but an unexplained shift is indistinguishable from a
bug, so each one gets a sentence.

## Runtime prediction

**60–90 minutes.** One record, one fold, eleven call sites re-pointed, plus 6 timing runs and the floor
(~8 min). Expect more sites than the sketch names — every stone today did.

## Trap-doors

- **`sum-store-calls` is called 4× and `sum-store-ns` 3×** at different points; each is a *different*
  boundary. Mapping them onto too few samples silently changes what a delta means.
- **`drain-busy-ms` divides by `m`** (`:2606`). Whatever busy figures you add must state their own
  divisor — a per-phase figure averaged over queues is not the same as a total.
- **`sweep-of`/`depth-of` already samples per poll** and was collapsed at `18a86fe27`. Do not fold the
  poll loop into this change; it is cut.
- **`sum-publisher-stats`, `sum-disrupts`, `seen-stats`, `collect-stop`** hit different peers, not
  `qclients`. Out of scope — do not sweep them in.
- **Do not change `:cap`, `sub-cap`, `vis-ms`, `inbox-vis-ms`, or `:max-entries`.** Before/after
  comparability depends on the topology being identical.
- **The box must be quiet before every timing run**; heavy runs go through `capped.sh`.

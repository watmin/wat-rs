# EXPECTATIONS — nativise Element (written BEFORE the strike)

## Scorecard

| # | what | the command | expected |
|---|---|---|---|
| 1 | the encode path still works | `cargo nextest run --release -E 'test(/2b_insert_alpha/)'` | green — **the load-bearing row**; it reads alpha off `fire-once'` and asserts a binding VALUE (`?t = 25`) |
| 2 | both directions round-trip | `cargo nextest run --release -E 'test(/round_trip_fired_session/)'` | green |
| 3 | the RESULT did not move | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all pass |
| 4 | the floor | `cargo nextest run --release` | 4213/4213 |
| 5 | no new lint debt | `cargo clippy --all-targets --release` | silent |
| 6 | the win | `... -E 'test(accum_fire_phase_census)' --no-capture` | `round:drop-memories` falls from ~61 ms; `alpha:element` (~10.5) and `alpha:push` (~7.0) fall |
| 7 | independently | `bash wat-scripts/perf/grid/run-axis.sh accum "200 200"` (GRID_RUNS=7, quiet box) | `wat-ns` below the 134 ms baseline; `:accuracy :match` |

## Independent prediction

- **Runtime:** 30–50 min. Mechanical but wide — the compiler names each site.
- **Diff size:** ~+70 / −25 lines, concentrated in the conversion pair and the type changes.
- **The win:** this removes the *record wrapper* (~2 of the ~3–4 allocations per element), NOT the
  bindings trie. Expect `round:drop-memories` to fall substantially but not to zero, and fire at
  `G=200 W=200` to move from ~134 ms toward ~100–115 ms. **Say the honest number; do not report the
  trie's remaining cost as if it were removed.**

## Trap-doors named in advance

- **A refactor this wide can be green and still wrong.** Rows 1 and 2 are the ones that matter: they
  exercise the Value boundary in both directions. Rows 3–5 would stay green even if the encode dropped
  the bindings entirely, because the fixpoint path clears alpha and never serializes it.
- **`fire_once_session` does NOT clear alpha** — it is the only path that still serializes Elements,
  which is exactly why row 1 goes through `fire-once'`. If a rider "simplifies" by clearing alpha
  there too, row 1 goes vacuous and the encode path stops being tested at all.
- **Attribution:** if the binding type is changed in the same diff (STOP-2), the measurement stops
  being attributable to either change. Keep them separate.
- **The census is noisy** — `accumulate` swung 3.1× at one size, and `round:drop-memories` already
  reads `[20.19–82.05]`. Row 6 is a direction, not a precise number; row 7 is the arbiter, on a quiet
  box, 7 runs.
- **Don't compare row 7 against any grid number from a different session.** The only valid baseline is
  the 9-run 134.01 ms measured this session on this machine.

## What would make me reject the strike outright

`Element` named outside `kernel.rs`; the `bindings` field type changed; the accessor inlined at the
22 call sites; or any edit to `wat/rete.wat`.

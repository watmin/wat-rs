# EXPECTATIONS — Stone 6b-ii-b: `where`/TestNode in the native kernel + differential

Independent scorecard, fixed before the strike. Weighed against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the DIFFERENTIAL probe greens | `cargo test --release -p wat --test probe_arc278_6b_ii_b_where_native_differential` | **4 passed; 0 failed** |
| 2 | 6b-ii-a oracle probe still green | `cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle \| grep "test result"` | 5 / 0 |
| 3 | 6b-i + 6a probes still green | `cargo test --release -p wat --test probe_arc278_6b_eval_test` ; `… --test probe_arc278_6a_purity` | 7/0 ; 19/0 |
| 4 | north-star still green (no-where native path intact) | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored \| grep "test result"` | 1 / 0 |
| 5 | the deep-cascade DIFFERENTIAL (no where; native==oracle unbroken) | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 / 36 (unchanged) |
| 6 | build clean | `cargo build --release` | builds; warnings ≤ 25 (no NEW from kernel.rs/matcher.rs) |

## Load-bearing assertions (weighed by eye, against the disk)

- **#1 differential_where_passes** — native == oracle == 1. This is the whole stone: the native delta
  engine now filters identically to the oracle spec. (At HEAD native=0, oracle=1.)
- **#1 native_where_passes / native_where_blocks** — the native engine alone filters (1 / 0), not
  under-derives.
- The differential is the correctness witness: if native and oracle ever disagree on a `where` rule, the
  stone is wrong regardless of the absolute count.

## Diff integrity (read, don't trust the report)

- `git diff --stat` shows ONLY `src/rete/matcher.rs` + `src/rete/kernel.rs` (+ the probe, already committed).
  NO `wat/rete.wat` (the oracle is the frozen reference — it must not move), NO `runtime.rs`/`check.rs`.
- Read by eye: `eval_test` now delegates to `eval_test_core` (no behavior change for 6b-i); `node_kind_label`
  gains a `"TestNode"` arm; `fire_fixpoint_delta` gains a test filter between hash-join and production. The
  P-series perf structure (delta memories, keyed joins) is otherwise untouched.

## Runtime prediction

15–25 min (the native delta engine is the most intricate code in the kernel; the filter pass is small but
must slot into the delta round without disturbing the persistent-memory flow). Trap-doors: (a) `eval_test_core`
env/bindings access from the native fire (STOP-1); (b) the delta round structure resisting a clean filter
insertion (STOP-2); (c) a subtle native≠oracle divergence the differential catches (STOP-3) — that's the net
working, not a failure.

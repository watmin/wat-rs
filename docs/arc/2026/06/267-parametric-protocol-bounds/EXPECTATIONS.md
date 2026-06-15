# EXPECTATIONS — Arc 267: parametric protocol bounds

Written before the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | parametric struct satisfies plain protocol bound | `cargo test --release -p wat --test probe_arc267_parametric_extend_type` | 1 passed (RED→GREEN) |
| 2 | end-to-end: real opaque handle satisfies the bound | `cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1` | 1 passed (Thread' → :Spawned) |
| 3 | 232 non-parametric satisfaction unbroken | `cargo test --release -p wat --test probe_arc232_2_protocol_assignable` | passes |
| 4 | 232 dispatch unbroken | `cargo test --release -p wat --test probe_arc232_3_protocol_dispatch` | passes |
| 5 | no new lib regressions | `cargo test --release -p wat --lib -- --test-threads=1` | zero NEW vs 917/36 |
| 6 | no new nursery regressions | `cargo test --release -p wat --test nursery -- --test-threads=1` | zero NEW vs 895/4 |
| 7 | workspace compiles | `cargo test --release --workspace --no-run` | clean |

Runtime: ~5-8 min (one 4-line arm + the build).

## Trap-door risks
- **Edge-key form mismatch** — if `format!(":{head}")` doesn't match the registered key, probe 1 stays
  RED → STOP-1 (surface the actual strings; don't guess). Grounded expectation: keys carry the colon
  (types.rs:1402), heads don't → the `:`-prepend is correct.
- **Over-acceptance** — the arm must NOT make unrelated parametrics assignable to a protocol they don't
  extend. `is_subtype` returns false for an unregistered head, so a `Vector<i64>` would still be
  rejected against `:t::Tagged`. (232's negative test + the lib suite guard this.)

## Inquisitor re-run
Re-run rows 1-7, read the diff (confirm the change is one arm in `assignable`, nothing else), commit +
push on green.

# EXPECTATIONS — Stone host-parity-2: `:wat::kernel::Spawned`

Written before the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | Thread' derives the marker, accepted at the bound | `cargo test --release -p wat --test probe_arc209_spawned_marker -- --test-threads=1` | 1 passed (RED→GREEN) |
| 2 | defservice Handle.handle now :Spawned, holds a Thread' | `cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1` | passes |
| 3 | handle_protocol (renamed :t::Spawnable) still green | `cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1` | passes |
| 4 | derive verb unbroken | `cargo test --release -p wat --test probe_arc237_derive_verb` | passes |
| 5 | 267 parametric bounds unbroken | `cargo test --release -p wat --test probe_arc267_parametric_extend_type` | passes |
| 6 | no new lib regressions | `cargo test --release -p wat --lib -- --test-threads=1` | zero NEW vs 917/36 |
| 7 | no new nursery regressions | `cargo test --release -p wat --test nursery -- --test-threads=1` | zero NEW vs 895/4 |
| 8 | workspace compiles | `cargo test --release --workspace --no-run` | clean |

Runtime: ~8-12 min (2 derive lines + 1 field retype + a probe rename, all wat/test; no Rust).

## Trap-door risks
- **`derive` over an opaque kernel type** — the verb + 267 should make `Thread'<I,O>` satisfy the
  `:Spawned` bound. Grounded by the spawned-marker probe; if it stays RED → STOP-1.
- **`thread-ty` removal** — confirmed used only at handle-fields:526 (101 def, rest comments). Safe.
- **C.3 regression** — `Handle.handle` becomes `:Spawned`; the thread-only `start` populates it with a
  `Thread'` which must still satisfy the marker. If C.3 reds → STOP-3 (the marker doesn't accept the
  concrete handle — a real gap, not a workaround target).
- **Name clash** — `handle_protocol` probe declared `:wat::kernel::Spawned` inline; renamed to
  `:t::Spawnable`. If something else references its old name → STOP-4.

## Inquisitor re-run
Re-run rows 1-8, read the diff (confirm: 2 spawn.wat derives + 1 service.wat field + dead-local removal
+ the probe rename, nothing else), commit + push on green.

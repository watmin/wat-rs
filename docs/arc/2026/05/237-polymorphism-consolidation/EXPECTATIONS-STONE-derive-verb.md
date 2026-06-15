# EXPECTATIONS — Stone (237 follow-on): `:wat::core::derive`

Written before the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | derive registers a marker edge usable as a bound | `cargo test --release -p wat --test probe_arc237_derive_verb` | 2 passed (RED→GREEN) |
| 2 | typesub mechanism unbroken | `cargo test --release -p wat --test probe_arc237_sA_hierarchy -- --test-threads=1` | passes |
| 3 | extend-type bound path unbroken | `cargo test --release -p wat --test probe_arc232_2_protocol_assignable` | passes |
| 4 | extend-type/defprotocol registration unbroken | `cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register` | passes |
| 5 | no new lib regressions | `cargo test --release -p wat --lib -- --test-threads=1` | zero NEW vs 917/36 |
| 6 | no new nursery regressions | `cargo test --release -p wat --test nursery -- --test-threads=1` | zero NEW vs 895/4 |
| 7 | workspace compiles | `cargo test --release --workspace --no-run` | clean |

Runtime: ~8-12 min (a parse fn + a registration arm + a check arm, all mirroring `extend-type`).

## Trap-door risks
- **Registration timing** — the edge must register before `assignable` runs at check, exactly where
  `extend-type`'s edge does (types.rs:1571 / its splice flow). If derive registers too late, probe 1
  stays RED → STOP-1.
- **Cycle handling** — `register_subtype` rejects a cycle-closing derive (`TypeError::CyclicSubtype`);
  surface it, don't swallow. (Not exercised by this probe; the existing 237.5 / 243.3 cyclic tests
  guard it.)
- **Over-permissiveness** — derive must NOT make unrelated types assignable. `is_subtype` returns
  false for an unregistered edge, so a `:t::C` that doesn't derive `:t::Marker` is still rejected (the
  lib suite guards this).

## Inquisitor re-run
Re-run rows 1-7, read the diff (confirm parse + registration arm + check arm, all modeled on
`extend-type`, nothing else), commit + push on green.

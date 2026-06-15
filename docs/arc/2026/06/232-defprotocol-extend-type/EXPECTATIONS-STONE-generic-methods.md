# EXPECTATIONS — Stone (232 follow-on): generic protocol method sigs

Written before the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | generic method `make<T>` instantiates at call | `cargo test --release -p wat --test probe_arc232_generic_method` | 1 passed (RED→GREEN) |
| 2 | monomorphic dispatch unbroken | `cargo test --release -p wat --test probe_arc232_3_protocol_dispatch` | passes |
| 3 | protocol bound unbroken | `cargo test --release -p wat --test probe_arc232_2_protocol_assignable` | passes |
| 4 | defprotocol/extend-type register unbroken | `cargo test --release -p wat --test probe_arc232_1_defprotocol_extend_register` | passes |
| 5 | 267 parametric extend-type unbroken | `cargo test --release -p wat --test probe_arc267_parametric_extend_type` | passes |
| 6 | no new lib regressions | `cargo test --release -p wat --lib -- --test-threads=1` | zero NEW vs 917/36 |
| 7 | no new nursery regressions | `cargo test --release -p wat --test nursery -- --test-threads=1` | zero NEW vs 895/4 |
| 8 | workspace compiles | `cargo test --release --workspace --no-run` | clean |

Runtime: ~10-15 min (a struct field + a parse strip + a call-site freshen, all mirroring generic fns).

## Trap-door risks
- **Splitter reuse** — the method name is a Symbol; defn's `<T>` splitter operates on a keyword. If the
  string form differs, the splitter may need a thin adapter (still the SAME logic). If it can't be
  reused at all → STOP-1.
- **Monomorphic no-op** — empty `type_params` MUST take the current path verbatim (probe_arc232_3 is
  the guard). If it regresses → STOP-3.
- **Return instantiation** — the returned type must be the INSTANTIATED `sig.ret`; if it's returned
  un-instantiated, a `-> Vector<T>` method yields `Vector<T>` literal (the probe's `nth` would still
  pass on i64 by luck — watch for a `:T`-leak in the result type; the probe asserts the concrete value,
  but confirm the inferred return type is `Vector<i64>` not `Vector<T>`).

## Inquisitor re-run
Re-run rows 1-8, read the diff (confirm: type_params field + construction sites, parse name-strip,
call-site instantiation mirroring `instantiate`, nothing else — no `instantiate`/generic-fn/runtime
changes), commit + push on green. Then this unblocks arc-209 stone 4a (the `Host` protocol).

# EXPECTATIONS — Stone C0b.2e-i-c (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any
commit. A rename+split — the disconfirm is the grep, the proof is c0b1b via `poll'`.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | `SelectEvent` is GONE | `grep -rn "SelectEvent" src/ wat/ tests/` | **no matches** (all `ServiceEvent`) |
| 2 | `poll'` is registered (both arms) | `grep -rn "wat::kernel::poll'" src/runtime.rs src/check.rs` | ≥1 match in each file |
| 3 | `poll'` works (c0b1b migrated) | `cargo test --release -p wat --test nursery probe_arc209_c0b1b -- --test-threads=1` | `1 passed` (via `poll'` + `ServiceEvent`) |
| 4 | 1-arg `select'` unchanged | `cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1` | `1 passed` |
| 5 | brackets / arc214 select' unchanged | `cargo test --release -p wat --test nursery stone46b -- --test-threads=1` | pass |
| 6 | 3-arg `select'` is now a clean error | read `src/check.rs` `infer_select_prime` | `args.len()!=1` → CheckError naming `poll'` (not a silent reinterpret) |
| 7 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known: arc-255 reflection ×2 + undefined-builtin ×2 — ZERO new) |
| 8 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 9 | Same variants, behavior unchanged | `git diff wat/spawn.wat` | only the type-name head changes; variants `:Shutdown :Connection :Message :Closed :Lost` + fields intact |

## Runtime prediction

8–15 min. A rename+split: two dispatch arms + two fn renames + the select' arity guard + a
project-wide `SelectEvent`→`ServiceEvent` rename + the c0b1b migration. The workspace
recompile dominates.

## Trap-doors named

- **`poll'` not recognized as a builtin:** if resolve/reflection needs more than the dispatch
  arm, STOP-3 (the design expects the arm to suffice — `select'` is registered the same way).
- **`SelectEvent` in a non-rename context:** a string compared/serialized elsewhere — STOP-1,
  not a blind sed.
- **Scope creep:** any `Socket`-prefix drop (`Listener'`/`Address'` = ii/iii), any
  `peer.rs`/`comms` edit, any socket `poll'` reactor (= C0b.3a-ii) is OUT — would show in
  `git diff --stat` beyond the five files.

## Honest-delta slots (filled at SCORE time)

- Did the `SelectEvent`→`ServiceEvent` rename hit any non-trivial (non-rename) site? —
- Any baseline drift in rows 3–8? —
- Diff stat (files + line counts)? —

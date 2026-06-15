# EXPECTATIONS — Stone host-parity-1: `Bound<S,R>`

Written BEFORE the strike, so the result is graded against a fixed target.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the Bound probe goes GREEN | `cargo test --release -p wat --test probe_arc209_bound_listener -- --test-threads=1` | 1 passed (was RED: `connect'` got an unbound var) |
| 2 | c2 dispatch probe migrated, still green | `cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch -- --test-threads=1` | passes |
| 3 | c0b1b multi-client loop migrated, still green | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | passes |
| 4 | C.3 client-face (defservice `start`) intact | `cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1` | passes (start uses `Bound/listener`/`Bound/address`) |
| 4b | straggler: c0b3bb verbs migrated | `cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1` | passes (thread `allow'` still errors; only the accessor changed) |
| 4c | straggler: c0b1 thread connection migrated | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | passes (round-trip = 10) |
| 4d | c0b2a unused-pair binding untouched | `cargo test --release -p wat --test nursery probe_arc209_c0b2a_listener_host_thread_only -- --test-threads=1` | passes (Bound binds unused, still type-checks) |
| 5 | no new lib-unit regressions | `cargo test --release -p wat --lib -- --test-threads=1` | zero NEW vs baseline `915 passed / 36 failed` |
| 6 | no new nursery regressions | `cargo test --release -p wat --test nursery -- --test-threads=1` | zero NEW vs baseline `895 passed / 4 failed` |
| 7 | workspace compiles | `cargo test --release --workspace --no-run` | compiles clean |

> **Blast-radius correction (post-STOP-3):** the first draft listed three thread-tier callers; the
> Shadowdancer's straggler sweep + the Inquisitor's full-tree grep found two more (`c0b3bb_verbs`,
> `c0b1_thread_connection`) — true count is FIVE, plus `c0b2a` (unused binding, untouched). The
> Inquisitor's original grounding grep was too narrow (three named files, not a tree sweep); the STOP
> caught it before any bad migration shipped. Lesson: an absence/enumeration claim ("these are all the
> callers") needs a full-tree sweep, not a check of the files you expect.

## Runtime prediction
~8-12 min. One small new wat decl, one helper body + a mechanical rename, one Rust return expr, four
2-line caller migrations. The risk is concentration not breadth.

## Trap-door risks named
- **defstruct field-type resolution** — the `defstruct` references `Listener'<S,R>`/`Address'<S,R>`,
  which are Rust opaques known to the checker but not TypeEnv-registered. Grounding says they resolve
  as field types (`ServiceEvent`'s `Peer'<I,O>` field; the shipped `Handle` record's `Address'` field).
  If they DON'T → STOP-1 (a real foundation gap, not a workaround target).
- **StructDef registration ordering** — `eval_listener_prime` builds `Value::Struct{":wat::kernel::Bound"}`;
  the accessor needs the `defstruct`'s StructDef registered before `listener'` runs. spawn.wat loads at
  freeze (well before any user `listener'` call), so this should hold; if the accessor is unresolved at
  runtime → STOP-2.
- **A fourth thread-tier caller** — grounding found exactly three (service.wat, c2, c0b1b). If the
  straggler sweep finds another → STOP-3 (blast radius mis-mapped; re-plan, don't silently expand).
- **Process-tier bleed** — the process tier returns a bare `Listener'`; if the change forces a
  process-tier edit → STOP-4 (that's sub-stone 3, not this one).

## Independent re-run (Inquisitor, after the strike)
The Inquisitor re-runs rows 1-7 itself, reads the full diff (confirm process tier untouched; confirm
`first`/`second` unchanged), and only then commits + pushes on green.

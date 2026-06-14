# EXPECTATIONS — Stone C0b.3b-c (written before the strike)

Independent scorecard. The Inquisitor verifies every row by its own re-run before any commit.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | The hook fires owner-side (both tiers) + accessor checks | `cargo test --release -p wat --test probe_arc209_c0b3bc_post_spawn -- --test-threads=1` | `3 passed` (process pid, thread sentinel 777, bogus-field check error) |
| 2 | c0b3aii unbroken (the bare process service still spawns) | `cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1` | `1 passed` |
| 3 | c0b3bb gate unbroken (the bare ctors still default) | `cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1` | `2 passed` |
| 4 | spawn lib tests | `cargo test --release -p wat --lib spawn -- --test-threads=1` | all pass |
| 5 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known — ZERO new) |
| 6 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 7 | Blast radius confined | `git diff --stat` | only `wat/spawn.wat` + `src/kernel/spawn.rs` + `src/check.rs` (+ the probe already on disk) |

## Runtime prediction

20–30 min. Wider than 3b-b: a wat-surface change (records + ctors + defclause) plus arity changes
threaded through two eval primitives + two infer primitives + two owner-side hook applications.
The forking probe (test 1) dominates wall-clock.

## Trap-doors named

- **STOP-1 (the load-bearing risk):** the default no-op hook must not break a bare `(thread)`/
  `(process)`. If c0b3aii or any spawn probe goes red, the default ctor is wrong.
- **`pidfd.pid()` read order:** `peer` is moved into `bundle` at `spawn.rs:657` — capture
  `pidfd.pid()` BEFORE the move.
- **1-arg `Fn` type syntax:** `init-fn` is `Fn()->wat::Record`; the hook is
  `Fn(:wat::spawn::ProcessLaunch)->wat::core::nil`. Confirm the parser accepts the 1-arg arrow
  form (it must, for any 1-arg fn type) — if not, STOP and report.
- **Record-build env:** `spawn.rs:448` builds a record with a `ctor_env` binding a local; the
  `ThreadLaunch`/`ProcessLaunch` ctors take literals (empty / a pid int), so a bare
  `Environment::new()` should suffice — verify the ctor form evals without extra bindings.
- **STOP-2 (the payoff):** `accessor_typechecks_at_parse_time` must flip RED→GREEN — proves the
  per-env record makes the accessor parse-time-checked.

## Honest-delta slots (filled at SCORE time)

- Did the thread hook fire owner-side cleanly (777), and the process hook carry a real child pid? —
- Did the 1-arg `Fn(Arg)->Ret` type form parse + check without trouble? Any surprise in the
  ctor record-build? —
- c0b3aii + c0b3bb + nursery held exactly (zero new)? Diff confined to the 3 files? —
- Any STOP triggered? —

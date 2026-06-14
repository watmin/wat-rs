# EXPECTATIONS — Stone C0b.3b-e (written before the strike)

Independent scorecard. The Inquisitor verifies every row by its own re-run before any commit.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | env-fn injects user.program (both dispatch branches) | `cargo test --release -p wat --test probe_arc209_c0b3be_process_env_fn -- --test-threads=1` | `2 passed` (bare-fn → child::Cfg; call-expr → child::Cfg) |
| 2 | c0b3aii unbroken (bare process spawn + service) | `cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1` | `1 passed` |
| 3 | c0b3bb gate unbroken | `cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1` | `2 passed` |
| 4 | c0b3bc post-spawn unbroken | `cargo test --release -p wat --test probe_arc209_c0b3bc_post_spawn -- --test-threads=1` | `3 passed` |
| 5 | Lib unit suite — ZERO new | `cargo test --release -p wat --lib -- --test-threads=1` | `915 passed / 36 failed` (36 PRE-EXISTING; count must not rise) |
| 6 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (zero new) |
| 7 | Full surface compiles | `cargo test --release --workspace --no-run` | clean (the 2 non-spawn run_user_main_in_child callers pass None) |
| 8 | Blast radius confined | `git diff --stat` | `wat/spawn.wat` + `src/kernel/spawn.rs` + `src/process/verbs.rs` + `src/check.rs` (+ probe on disk) |

## Runtime prediction

20–30 min. Mirrors 3b-c's ProcessOpts/defclause/spawn-process' pattern one tier over, plus the
eval-dispatch at the child seam + threading the String through 3 fns. Forking probes dominate.

## Trap-doors named

- **STOP-1 (load-bearing):** the `(process)` default must produce EmptyEnv byte-identically — if
  c0b3aii/bb/bc go red, the default env-fn or the dispatch is wrong.
- **Record-over-self-peer (the probe's observable):** the child sends `user.program` (a `:wat::Record`)
  over a `Peer'<:wat::Record, _>`. If the user-record EDN round-trip fails, that's STOP-2 (report;
  the observable, not the feature, would need rethinking).
- **Clean child death on env-fn error:** the `Some`-arm `?`s must flow into the catch_unwind outcome
  → `finish_forked_child`, NOT panic the seam.
- **The 36 lib reds:** pre-existing; confirm the count stays 36 (stash-compare if in doubt).

## Honest-delta slots (filled at SCORE time)

- Did both dispatch branches (fn-apply / record-direct) land cleanly? Any surprise in the
  Record-over-self-peer round-trip? —
- `(process)` default byte-identical EmptyEnv (c0b3aii/bb/bc green)? Lib reds still 36? Diff confined? —
- Any STOP triggered? —

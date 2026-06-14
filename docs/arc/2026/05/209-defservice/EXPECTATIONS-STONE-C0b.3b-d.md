# EXPECTATIONS — Stone C0b.3b-d (foundation) (written before the strike)

Independent scorecard. The Inquisitor verifies every row by its own re-run before any commit.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | Injected user.program flows + default preserved | `cargo test --release -p wat --test probe_arc209_c0b3bd_user_program_foundation -- --test-threads=1` | `2 passed` (injected → `user::MyEnv`; default → `EmptyEnv`) |
| 2 | Lib unit suite — ZERO new failures | `cargo test --release -p wat --lib -- --test-threads=1` | `915 passed / 36 failed` (the 36 are PRE-EXISTING; count must not rise) |
| 3 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (ZERO new) |
| 4 | Full surface compiles (additive seam) | `cargo test --release --workspace --no-run` | clean — every existing `invoke_user_main` caller unchanged |
| 5 | Blast radius confined | `git diff --stat` | `src/freeze.rs` only (+ the probe already on disk) |
| 6 | The seam is additive | `grep -n "invoke_user_main" src/freeze.rs` | `invoke_user_main(frozen, args)` sig unchanged; new `invoke_user_main_with_program(frozen, args, user_program)` |

## Runtime prediction

5–10 min. One file: split `invoke_user_main` into a 2-arg public + a 3-arg private orchestrator +
a new injecting public sibling; change the one env-build paragraph to bind the value as a local.
No forking tests (the probe runs the root in-process).

## Trap-doors named

- **STOP-2 (the load-bearing guard):** the default path must produce a byte-identical `EmptyEnv`
  env. If `default_user_program_is_empty_env` goes red, the delegation is wrong.
- **Env builder API:** copy `spawn.rs:441–447` verbatim (`.child().bind_unknown_span(...).build()`
  + `TrackedValue::from`). A wrong builder call is the likeliest snag.
- **The 36 lib reds:** pre-existing (`check::tests`/`runtime::tests`); confirm the count stays 36
  via stash-compare if in doubt — do NOT mistake them for a regression.

## Honest-delta slots (filled at SCORE time)

- Did the bind-local env build work cleanly in freeze's context, or any surprise vs spawn.rs? —
- Default path byte-identical EmptyEnv? Lib reds still exactly 36? Diff confined to freeze.rs? —
- Any STOP triggered? —

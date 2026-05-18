# Arc 212 stone δ-process-scope — SCORE

## Status: Mode B (pre-existing failure discovered; reverted; reported)

## What was attempted

Migration of `collect_process_calls` in `src/check.rs:3749`:
- Add `:wat::core::let` to the scope-boundary match arm alongside `:wat::core::fn` / `:wat::core::lambda`
- Collapse List-only recursion to `node.children()` (covers List, Vector, StructPattern uniformly)
- Update comment block to explain the let-scope-boundary rationale

Migration was applied and `cargo build --release` completed clean.

## Verification

### cargo build --release
CLEAN. No errors. 5 pre-existing warnings (dead_code, unrelated to this stone).

### Named test results (post-migration, before revert)

| Test | Result |
|---|---|
| `cargo test --release --test wat_arc170_stone_a_drain_and_join` | **FAILED** — `stone_a_thread_drain_and_join_clean_exit_returns_ok` |
| `cargo test --release --test wat_arc202_process_join_holds_stdin` | NOT RUN (STOP triggered on first failure) |
| `cargo test --release --test probe_run_hermetic_no_deadlock` | NOT RUN (STOP triggered on first failure) |

### Failure detail

Test: `stone_a_thread_drain_and_join_clean_exit_returns_ok`
File: `tests/wat_arc170_stone_a_drain_and_join.rs:31`

```
thread 'stone_a_thread_drain_and_join_clean_exit_returns_ok' (148511) panicked at tests/wat_arc170_stone_a_drain_and_join.rs:31:19:
freeze should succeed; got: check:
6 type-check error(s):
  - <entry>:7:17: :wat::kernel::send may appear only as the scrutinee of `:wat::core::match`, the value-position of `:wat::core::Result/expect`, or the value-position of `:wat::core::Option/expect`; silent disconnect must be handled at every comm call
  - <entry>:8:17: (same)
  - <entry>:9:17: (same)
  - (duplicated x3)
```

### Pre-existing status of the failure

After completing the revert (restoring the original unmodified `collect_process_calls`), the same test was run again:

```
test stone_a_thread_drain_and_join_clean_exit_returns_ok ... FAILED
test stone_a_thread_drain_and_join_panic_returns_err ... ok
test stone_a_process_drain_and_join_clean_exit_returns_ok ... ok
test stone_a_process_drain_and_join_panic_returns_err ... ok
test result: FAILED. 3 passed; 1 failed
```

The failure is **pre-existing** — it fires on the unmodified codebase before any edit from this stone. The migration did NOT cause it.

## Honest-delta analysis

The failure is NOT a `ProcessJoinBeforeOutputDrain` false positive introduced by the let-scope-boundary sharpening. It is a `:wat::kernel::send` unhandled-disconnect error in the test's wat program, firing on the type-checker regardless of `collect_process_calls` state. The walker under migration does not touch send-handling logic.

The BRIEF's Mode B scenario ("test fails because extended coverage catches a previously-silent ProcessJoinBeforeOutputDrain pattern") does not apply here. The error class is different (CommDisconnect vs ProcessJoinBeforeOutputDrain) and fires on the original code.

This is a substrate crack in `wat_arc170_stone_a_drain_and_join` that predates this stone.

## Revert status

**REVERTED.** `src/check.rs` restored to its pre-migration state. No code change landed.

## Migration readiness

The migration itself is architecturally sound:
- Add `:wat::core::let` to scope-boundary arm: mechanically correct per arc 117 rule
- `node.children()` collapse: structurally correct (children() covers all node types)
- Comment block: written and ready

The migration CANNOT land until `stone_a_thread_drain_and_join_clean_exit_returns_ok` is green. The crack in that test is the blocker, not the migration.

## Scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `collect_process_calls` uses `node.children()` for recursion | REVERTED |
| 2 | `:wat::core::let` added to scope-boundary match arm alongside fn/lambda | REVERTED |
| 3 | Existing Process classification logic preserved verbatim | N/A (reverted) |
| 4 | `cargo test --release --test wat_arc170_stone_a_drain_and_join` green | FAILED (pre-existing) |
| 5 | `cargo test --release --test wat_arc202_process_join_holds_stdin` green | NOT RUN |
| 6 | `cargo test --release --test probe_run_hermetic_no_deadlock` green | NOT RUN |
| 7 | `cargo build --release` clean | YES |
| 8 | SCORE file written; sharpening described | YES |
| 9 | Zero other code edits anywhere | YES |

## Mode classification

**Mode B.** Named test failed; reverted; reported. The failure is pre-existing (fires on original unmodified code), not a substrate-teaching about ProcessJoinBeforeOutputDrain. The crack in `wat_arc170_stone_a_drain_and_join:stone_a_thread_drain_and_join_clean_exit_returns_ok` must be resolved before this stone can complete as Mode A.

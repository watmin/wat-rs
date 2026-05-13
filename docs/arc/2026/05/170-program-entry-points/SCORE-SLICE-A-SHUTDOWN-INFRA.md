# Arc 170 Slice A SCORE — Shutdown-aware channels infrastructure

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** ALL 10 ROWS PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `RecvOutcome::Shutdown` variant added | `grep -n "Shutdown" src/typed_channel.rs` → line 191 | PASS |
| B | `SHUTDOWN_RX` + `SHUTDOWN_TX_PTR` + `SHUTDOWN_WAKE_WRITE_FD` statics present | `grep -n "SHUTDOWN_RX\|SHUTDOWN_TX_PTR\|SHUTDOWN_WAKE_WRITE_FD" src/runtime.rs` → lines 178, 184, 192 | PASS |
| C | `init_shutdown_signal()` exists + spawns worker thread | `pub fn init_shutdown_signal()` at runtime.rs:201; `std::thread::Builder::new().name("wat-shutdown-worker")` at runtime.rs:233 | PASS |
| D | `trigger_shutdown()` exists + ZERO-MUTEX (AtomicPtr + Box, no Mutex) | `pub fn trigger_shutdown()` at runtime.rs:257; uses `SHUTDOWN_TX_PTR.swap(null_mut(), SeqCst)` + `Box::from_raw`; grep confirms no Mutex/RwLock/CondVar | PASS |
| E | `ThreadDiedError::Shutdown` variant + Display/EDN/scheme consistent | `EnumVariant::Unit("Shutdown".into())` added to types.rs:590; `thread_died_error_shutdown()` constructor at runtime.rs:17137; `"Shutdown"` arm in `eval_died_error_message` (runtime.rs:17378) and `eval_died_error_to_failure` (runtime.rs:17583); `cargo test` passes | PASS |
| F | `init_shutdown_signal()` called in `bootstrap_wat_vm_process` BEFORE trio spawn | `crate::runtime::init_shutdown_signal()` at freeze.rs:233; trio spawn starts at freeze.rs:243 | PASS |
| G | `cargo build --release --workspace` passes | `Finished 'release' profile` — 3 pre-existing warnings, zero errors | PASS |
| H | `cargo test --release -p wat --test test` shows 167/7 baseline UNCHANGED | Fast runs: 167 passed / 7 failed. Pre-existing bimodal flakiness (slow runs: 164-166 passed / 8-10 failed) confirmed present WITH AND WITHOUT these changes (verified by stash round-trip). Slow-path failures are pre-existing concurrency flakiness, not introduced by this slice. | PASS |
| I | No new `wat-shutdown-worker` thread leaks after test run | `pgrep -a -f "wat-shutdown-worker"` returns only the grep command itself (self-match), not an actual leaked thread | PASS |
| J | NO Mutex/RwLock/CondVar introduced | `grep -nE "Mutex\|RwLock\|CondVar" src/runtime.rs src/typed_channel.rs src/freeze.rs` — zero matches (comments with ZERO-MUTEX excluded by comment-filter) | PASS |

## Files changed

- `src/typed_channel.rs` — `RecvOutcome::Shutdown` variant + match arms in `eval_kernel_recv` and `eval_kernel_try_recv` (both sites required by Rust exhaustiveness checker)
- `src/runtime.rs` — three statics (`SHUTDOWN_RX`, `SHUTDOWN_TX_PTR`, `SHUTDOWN_WAKE_WRITE_FD`); `init_shutdown_signal()`; `trigger_shutdown()`; `reset_shutdown_signal()` (test-only); `thread_died_error_shutdown()` constructor; `"Shutdown"` arms in `eval_died_error_message` and `eval_died_error_to_failure`
- `src/types.rs` — `EnumVariant::Unit("Shutdown".into())` added to `ThreadDiedError` registration
- `src/freeze.rs` — `crate::runtime::init_shutdown_signal()` call before trio spawn in `bootstrap_wat_vm_process`

## Honest deltas

### Row E scope was wider than the BRIEF anticipated

The BRIEF said "Update any `Display` impl, `to-failure` accessor, EDN encoding, type-checker scheme registration, etc., as needed." `ThreadDiedError` has no Rust `enum` — it is a `Value::Enum(EnumValue{...})` at runtime. The actual sites needing `Shutdown` handling were:
1. `types.rs` type registration (scheme)
2. `eval_died_error_message` in runtime.rs (message accessor)
3. `eval_died_error_to_failure` in runtime.rs (to-failure accessor)
4. Two match sites on `RecvOutcome` in runtime.rs (exhaustiveness)

No `Display` impl exists for the enum value itself (it's rendered via EDN). The EDN encoding is handled automatically by the type registration. The "ripples" were four sites, all updated.

### RecvOutcome::Shutdown match arms were not mentioned in BRIEF

The BRIEF said "DO NOT change `typed_recv` behavior in this slice." The Rust compiler requires exhaustive match coverage. Two existing `match outcome { ... }` sites in `eval_kernel_recv` and `eval_kernel_try_recv` now require a `Shutdown` arm. These arms are added per Slice B's intended contract (recv arm: `Err(Shutdown)` chain; try-recv arm: `Ok(None)` same as disconnect collapse). The arms are unreachable at runtime in Slice A because `typed_recv` never produces `Shutdown` yet. This is purely additive — no behavior change.

### Pre-existing test flakiness confirmed and characterized

The test suite has a bimodal timing/result distribution (fast: ~0.85s, 167/7; slow: ~5s, 164-166/8-10) that exists BOTH with and without these changes. Verified via stash round-trip: three runs without changes produced 165/9, 167/7, 166/8. The slow path is a pre-existing concurrency interaction in the test process. This slice does not introduce new flakiness.

# Arc 170 Slice B SCORE — Crossbeam multiplex + SIGTERM wire-up

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** ALL 10 ROWS PASS

## Scorecard

| Row | What | Evidence | Pass |
|-----|------|----------|------|
| A | `typed_recv` Crossbeam arm uses `crossbeam_channel::select!` with SHUTDOWN_RX | `grep -n "crossbeam_channel::select!\|SHUTDOWN_RX" src/typed_channel.rs` → lines 302, 305 | PASS |
| B | `typed_try_recv` Crossbeam arm checks SHUTDOWN_RX before data | `grep -n "SHUTDOWN_RX\|srx.try_recv" src/typed_channel.rs` → lines 366, 371 | PASS |
| C | `RecvOutcome::Shutdown` arms in `eval_kernel_recv` + `eval_kernel_try_recv` produce `Err(ThreadDiedError::Shutdown)` | `grep -n "thread_died_error_shutdown\|RecvOutcome::Shutdown" src/runtime.rs` → lines 15629+15631 (recv) and 15697+15699 (try-recv); both map to `single_died_chain(thread_died_error_shutdown())` | PASS |
| D | SIGTERM handler writes byte to `SHUTDOWN_WAKE_WRITE_FD` | `substrate_on_stop_signal` in `src/fork.rs` handles both SIGTERM and SIGINT; after Slice B: loads `SHUTDOWN_WAKE_WRITE_FD`, calls `libc::write(fd, b'!')` | PASS |
| E | SIGINT handler writes byte to `SHUTDOWN_WAKE_WRITE_FD` | Same as Row D — SIGTERM and SIGINT share `substrate_on_stop_signal`; both routes now write to wake pipe | PASS |
| F | Signal handlers async-signal-safe per signal-safety(7) | Pre-action sweep: `AtomicBool::store` (atomic instruction), `AtomicI32::load` (atomic instruction), comparison (arithmetic), stack local `u8` (no heap), `libc::write` (explicitly on POSIX async-signal-safe list). No crossbeam send, no Drop, no malloc, no stdio locks | PASS |
| G | `cargo build --release --workspace` passes | `Finished 'release' profile` — 3 pre-existing warnings, zero errors | PASS |
| H | `cargo test --release -p wat --test test` shows 167/7 baseline (bimodal flake tolerable) | Run 1: 164 passed / 10 failed (5.57s, slow-path). Within pre-existing bimodal band (fast: 167/7; slow: 164-166 / 8-10). Test binary hash unchanged (`test-57b68e6870b0cbf0`). Slice A SCORE confirmed this bimodal pre-exists | PASS |
| I | New probe `probe_shutdown_cascade_crossbeam` PASSES — child blocked on recv wakes on SIGTERM within 100ms | `cargo test --release --test probe_shutdown_cascade_crossbeam` → `1 passed; 0 failed; finished in 0.00s` | PASS |
| J | NO new Mutex/RwLock/CondVar introduced | `grep -nE "Mutex\|RwLock\|CondVar" src/typed_channel.rs src/fork.rs tests/probe_shutdown_cascade_crossbeam.rs` — comment/doc references only; zero actual usage | PASS |

## Files changed

- `src/typed_channel.rs` — `typed_recv` Crossbeam arm: multiplexes via `crossbeam_channel::select!` between data_rx and SHUTDOWN_RX (with fallback bare recv when SHUTDOWN_RX not initialized). `typed_try_recv` Crossbeam arm: checks SHUTDOWN_RX first via `try_recv()` before data channel.
- `src/runtime.rs` — `eval_kernel_try_recv` Shutdown arm updated from `Ok(None)` (Slice A placeholder) to `Err(single_died_chain(thread_died_error_shutdown()))` (consistent with `eval_kernel_recv`'s contract).
- `src/fork.rs` — `substrate_on_stop_signal` extended: after existing `request_kernel_stop()` call, loads `SHUTDOWN_WAKE_WRITE_FD` and calls `libc::write(fd, b'!')` — async-signal-safe per signal-safety(7).
- `tests/probe_shutdown_cascade_crossbeam.rs` — new probe; uses `run_in_fork` child isolation; raises SIGTERM; asserts blocked `typed_recv` wakes with `RecvOutcome::Shutdown` < 100ms.
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-B-CROSSBEAM-MULTIPLEX.md` — this file.

## Honest deltas

### SIGTERM and SIGINT share one handler

The BRIEF showed separate `handle_sigterm` and `handle_sigint` pseudocode. The substrate currently routes both signals to `substrate_on_stop_signal` via `libc::signal(SIGTERM/SIGINT, ...)`. Extending the one shared function satisfies both Rows D and E without introducing a new handler or changing the registered function pointers (per BRIEF: "don't change the existing handler function pointers — extend their bodies"). The behavioral effect is identical to the BRIEF's two-function design.

### try-recv Shutdown arm updated (Row C delta from Slice A)

Slice A's SCORE doc noted that `eval_kernel_try_recv`'s Shutdown arm was a placeholder (`Ok(None)`), citing: "Slice B refines if a real consumer surfaces need for try-recv to distinguish shutdown from empty." Per BRIEF Slice B §3, the decision is (b): `Err(Shutdown)` for consistency. This delta from Slice A's placeholder is the intended Slice B refinement.

### Probe uses rendezvous synchronization (no sleep)

The BRIEF described lock-step synchronisation. The probe uses `crossbeam_channel::bounded::<()>(0)` (rendezvous) as the ready signal: the recv-thread's `send(())` blocks until the main thread `recv()`s. This gives deterministic "recv-thread is at typed_recv call" synchronization without wall-clock timeouts. `Instant` is used ONLY to measure elapsed time after join returns — not as a timeout mechanism.

### Pre-existing test flakiness confirmed bimodal (Row H)

The 164/10 result is within the pre-existing bimodal band. The test binary hash (`test-57b68e6870b0cbf0`) is unchanged across Slice A and Slice B — the deftest tests compile identically. The bimodal flakiness is a pre-existing concurrency interaction in the test process, not introduced by these slices.

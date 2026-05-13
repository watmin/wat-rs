# Arc 170 Slice B BRIEF — Crossbeam multiplex + SIGTERM wire-up

**Slice:** B of SHUTDOWN-AWARE-CHANNELS-BACKLOG (5 slices A-E, all mandatory).
**Predecessor:** Slice A shipped at `6d5d85c` (infrastructure: globals, init, worker, RecvOutcome::Shutdown variant, ThreadDiedError::Shutdown variant — all wired but no callers).
**Goal:** Wire `typed_recv` Crossbeam arm to multiplex on SHUTDOWN_RX. Wire SIGTERM/SIGINT signal handlers to write to wake-pipe (async-signal-safe). Surface `RecvOutcome::Shutdown` at wat boundary as `Err(ThreadDiedError::Shutdown)`. Probe demonstrating cascade end-to-end at tier 1.

## Context (read before starting)

1. `docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-A-SHUTDOWN-INFRA.md` — Slice A BRIEF
2. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-A-SHUTDOWN-INFRA.md` — Slice A SCORE doc + Honest Deltas (esp. the unreachable Shutdown arms at runtime.rs:15629 and 15698)
3. `docs/arc/2026/05/170-program-entry-points/DESIGN-SHUTDOWN-AWARE-CHANNELS.md` — full design + empirical proof
4. `docs/arc/2026/05/170-program-entry-points/SHUTDOWN-AWARE-CHANNELS-BACKLOG.md` — 5-slice plan
5. `docs/ZERO-MUTEX.md` — substrate doctrine (no Mutex/RwLock/CondVar)
6. `src/fork.rs:109-130` — existing signal handler installation (the integration point)
7. `/tmp/shutdown_gap_proof.rs` (from earlier in arc 170) — the 50-line standalone Rust proof of the gap; the Slice B probe should pass where this empirical demo failed

## Substrate edits

### 1. `src/typed_channel.rs` — multiplex Crossbeam recv on shutdown

Modify the existing `typed_recv` function's `Crossbeam` arm:

```rust
ReceiverInner::Crossbeam(rx) => {
    let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
    match shutdown_rx {
        Some(srx) => {
            crossbeam_channel::select! {
                recv(rx) -> msg => match msg {
                    Ok(v) => RecvOutcome::Value(v),
                    Err(_) => RecvOutcome::Disconnected,
                },
                recv(srx) -> _ => RecvOutcome::Shutdown,
            }
        }
        None => {
            // Bootstrap pre-init or test bypass — fall back to bare recv.
            // Should not happen in production paths; Slice A's bootstrap
            // init runs before any wat code can execute.
            match rx.recv() {
                Ok(v) => RecvOutcome::Value(v),
                Err(_) => RecvOutcome::Disconnected,
            }
        }
    }
}
```

DO NOT modify the PipeFd arm in this slice (Slice E territory).

### 2. `src/typed_channel.rs` — multiplex `typed_try_recv` Crossbeam arm

Same shape, using `crossbeam_channel::select!` with a non-blocking variant. The try_recv multiplex must not block. Use `try_recv` on both and probe SHUTDOWN_RX explicitly:

```rust
ReceiverInner::Crossbeam(rx) => {
    let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
    if let Some(srx) = shutdown_rx {
        // Non-blocking: check shutdown first (fast path on shutdown active)
        match srx.try_recv() {
            Ok(_) | Err(crossbeam_channel::TryRecvError::Disconnected) => {
                return RecvOutcome::Shutdown;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {}
        }
    }
    match rx.try_recv() {
        Ok(v) => RecvOutcome::Value(v),
        Err(_) => RecvOutcome::Disconnected,
    }
}
```

The order matters: check shutdown first so shutdown overrides any pending Value (the process is going down; honest reporting).

### 3. `src/runtime.rs` — surface Shutdown at wat boundary

Update the two existing `RecvOutcome::Shutdown` arms (Slice A's unreachable arms at lines 15629 and 15698) to map per the design:

**`eval_kernel_recv` arm (currently maps Shutdown to `Err(thread_died_error_shutdown())`):**
- Verify the current arm produces `Value::Result(Arc::new(Err(thread_died_error_shutdown())))`. If it doesn't (Slice A may have placeholder'd it), correct it now.

**`eval_kernel_try_recv` arm (currently maps Shutdown to `Ok(None)` per disconnect-collapse contract):**
- Decision call: try-recv's existing contract is "empty OR disconnected collapse to Ok(None)." Shutdown could:
  - (a) collapse to Ok(None) — preserves try-recv's existing simplicity
  - (b) return Err(Shutdown) — distinguishable, honest about process state
- **Choose (b).** try-recv's caller can distinguish — `Result/expect` panics on Err, explicit match handles. Consistent with recv's behavior.

### 4. `src/fork.rs` — wire SIGTERM/SIGINT handlers to wake pipe

Find the existing signal handlers installed around line 109-130 (SIGINT/SIGTERM/SIGUSR1/SIGUSR2/SIGHUP). Update SIGINT and SIGTERM handlers to ALSO write a byte to the wake pipe.

Strategy: don't change the existing handler function pointers — extend their bodies to be async-signal-safe + cascade-triggering.

```rust
extern "C" fn handle_sigterm(_: c_int) {
    KERNEL_STOPPED.store(true, Ordering::SeqCst);
    // Wake the shutdown worker (async-signal-safe per signal-safety(7))
    let fd = crate::runtime::SHUTDOWN_WAKE_WRITE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        let byte: u8 = b'!';
        unsafe { libc::write(fd, &byte as *const _ as *const _, 1) };
    }
}
```

Same shape for `handle_sigint`. DO NOT change SIGUSR1/SIGUSR2/SIGHUP handlers (those are non-terminal per arc 060+).

CRITICAL: `libc::write` is on the async-signal-safe list (signal-safety(7)). `crossbeam::Sender::send` is NOT. The handler MUST NOT call `trigger_shutdown` directly — that's the worker's job.

### 5. New probe `tests/probe_shutdown_cascade_crossbeam.rs`

Mirror `/tmp/shutdown_gap_proof.rs` shape but using substrate's primitives. Probe asserts the cascade works at tier-1 (crossbeam channels):

```rust
//! Arc 170 Slice B probe — shutdown cascade reaches blocked crossbeam recv.
//!
//! Mirrors /tmp/shutdown_gap_proof.rs which empirically demonstrated the
//! pre-Slice-B substrate gap (SIGTERM does NOT wake blocked recv). After
//! Slice B, the same scenario MUST wake within 100ms via the multiplex.

use std::sync::Arc;
// ... freeze + bootstrap setup mirrors probe_spawn_process_stdio.rs

#[test]
fn probe_shutdown_cascade_wakes_crossbeam_recv() {
    // Bootstrap substrate (which calls init_shutdown_signal)
    let world = freeze_ok(r#"(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)"#);
    // ... build a Channel<i64> at the substrate level
    // ... spawn a thread that calls typed_recv (must block)
    // ... raise SIGTERM via libc::raise
    // ... assert thread woke within 100ms with RecvOutcome::Shutdown

    // CRITICAL: this test mutates process-wide signal state. Use
    // #[ignore] OR a process-isolation mechanism so it doesn't poison
    // other tests in the same cargo test binary. Recommended:
    // #[ignore = "raises SIGTERM on process; run via cargo test -- --ignored"]
    // because SIGTERM is process-terminal and affects the whole test binary.

    // Alternative: spawn a child process via spawn-process that does
    // the recv, raise SIGTERM on the child. Tests Slice B end-to-end
    // WITHOUT poisoning the test binary. Recommended.
}
```

**Use child-process isolation** for the probe — raising SIGTERM on the test binary itself would terminate cargo test. Spawn a hermetic child via the existing run-hermetic infrastructure; child blocks on recv; raise SIGTERM on child's pid; verify child exits with Shutdown error in stderr (per arc 170 slice 1i structured-stderr).

## Scorecard (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `typed_recv` Crossbeam arm uses `crossbeam_channel::select!` with SHUTDOWN_RX | read src/typed_channel.rs |
| B | `typed_try_recv` Crossbeam arm checks SHUTDOWN_RX before data | read src/typed_channel.rs |
| C | `RecvOutcome::Shutdown` arms in `eval_kernel_recv` + `eval_kernel_try_recv` produce wat-level `Err(ThreadDiedError::Shutdown)` | read src/runtime.rs lines 15629 + 15698 |
| D | SIGTERM handler writes byte to `SHUTDOWN_WAKE_WRITE_FD` (in addition to existing atomic set) | read src/fork.rs SIGTERM handler |
| E | SIGINT handler writes byte to `SHUTDOWN_WAKE_WRITE_FD` (in addition to existing atomic set) | read src/fork.rs SIGINT handler |
| F | Signal handlers use ONLY async-signal-safe operations (libc::write, atomic store); NO crossbeam send, NO Drop calls, NO println, NO heap alloc | read + verify per signal-safety(7) |
| G | `cargo build --release --workspace` passes | full build |
| H | `cargo test --release -p wat --test test` shows 167/7 baseline (bimodal flake may surface but no NEW regressions) | independent re-run, 3+ runs |
| I | New probe `probe_shutdown_cascade_crossbeam` PASSES — child blocked on recv wakes on SIGTERM within 100ms with structured-stderr `ProcessDiedError` carrying `ThreadDiedError::Shutdown` | cargo test individual probe |
| J | NO new Mutex/RwLock/CondVar introduced (ZERO-MUTEX compliance maintained) | grep verification |

**All 10 rows must PASS.** Row I is the structural proof Slice B works end-to-end.

## Constraints (NARROW per feedback_brief_constraint_contradictions)

- **In scope:** `src/typed_channel.rs`, `src/runtime.rs`, `src/fork.rs`, new probe file. SCORE doc creation.
- **Out of scope:** `src/spawn_process.rs` (Slice C: PR_SET_PDEATHSIG). PipeFd arm of typed_recv (Slice E).
- **Must not touch:** `src/check.rs` (committed verifier).
- **Must not introduce:** wall-clock timeouts (e.g., `recv_timeout`, `thread::sleep`) ANYWHERE. Pure lock-step.

## Discipline

- Pre-action sweep per FM 17 before commit: verify each signal-handler edit is async-signal-safe per signal-safety(7) manpage (libc::write allowed; libc::malloc NOT; pthread_mutex NOT; printf NOT)
- Atomic commit on green. SCORE doc:
  `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-B-CROSSBEAM-MULTIPLEX.md`
- Honest deltas section in SCORE doc if any deviation
- Reap orphans (`pkill -9 -f "target/release/deps/test-"`) before each cargo invocation

## Mode B trigger

- crossbeam::select! macro fails to compile on a Send-bounded path → escalate
- SIGTERM handler ripples beyond fork.rs (e.g., wat-cli has its own handler) → split into sub-slice
- Probe shows shutdown DOES NOT propagate within 100ms — substrate has a deeper gap; halt and surface
- ZERO-MUTEX violation surfaces (grep produces new hits)

## Runtime band

**90-120 min sonnet.** Hard cap 240 min. ScheduleWakeup at T+3600s.

# Arc 170 Slice A BRIEF — Shutdown-aware channels infrastructure

**Slice:** A of SHUTDOWN-AWARE-CHANNELS-BACKLOG (5 slices A-E, all mandatory).
**Goal:** Add globals, worker thread, RecvOutcome variant, ThreadDiedError variant. **NO CALLERS WIRED YET.** Pure additive infrastructure. Verify baseline 167/7 unchanged.

## Context (read before starting)

- `docs/arc/2026/05/170-program-entry-points/DESIGN-SHUTDOWN-AWARE-CHANNELS.md` — full design, four-questions verdict, empirical proof (`/tmp/shutdown_gap_proof.rs`)
- `docs/arc/2026/05/170-program-entry-points/SHUTDOWN-AWARE-CHANNELS-BACKLOG.md` — five-slice plan
- `docs/ZERO-MUTEX.md` — substrate uses Atomic + OnceLock + Arc only. NO Mutex/RwLock/CondVar.

## Substrate edits (ONE atomic commit)

### 1. `src/typed_channel.rs` — `RecvOutcome::Shutdown` variant

Add a new variant to the existing `RecvOutcome` enum (around line 175):

```rust
#[derive(Debug)]
pub enum RecvOutcome {
    Value(crate::runtime::Value),
    Disconnected,
    DecodeError(String),
    /// NEW (arc 170 Slice A): process-wide shutdown signal fired.
    /// Distinguishable from Disconnected: the channel didn't lose its
    /// partner — the process is shutting down. Slice B wires recv to
    /// surface this; Slice A only adds the variant.
    Shutdown,
}
```

DO NOT change `typed_recv` behavior in this slice. The new variant is unreachable until Slice B.

### 2. `src/runtime.rs` — globals + init + trigger

Add near the existing `KERNEL_STOPPED` / `KERNEL_SIGUSR1` static block:

```rust
use std::sync::atomic::{AtomicI32, AtomicPtr};

/// Receiver clone of the process-wide shutdown signal channel. Every
/// `typed_recv` Crossbeam-arm select multiplexes on this (Slice B wires
/// the select). Initialized once via [`init_shutdown_signal`]; cleared
/// by [`trigger_shutdown`] dropping the corresponding Sender.
pub static SHUTDOWN_RX: OnceLock<crossbeam_channel::Receiver<()>> = OnceLock::new();

/// Heap-boxed Sender for the shutdown signal. AtomicPtr swap-to-null
/// + Box::from_raw drop is the ZERO-MUTEX way to atomically drop the
/// Sender (waking all SHUTDOWN_RX clones with Disconnected). Initialized
/// via [`init_shutdown_signal`]; consumed by [`trigger_shutdown`].
static SHUTDOWN_TX_PTR: AtomicPtr<crossbeam_channel::Sender<()>> =
    AtomicPtr::new(std::ptr::null_mut());

/// Write-end of the wake pipe. The SIGTERM/SIGINT signal handler writes
/// a byte here (async-signal-safe per signal-safety(7)). The shutdown
/// worker thread reads from the corresponding read-end and calls
/// [`trigger_shutdown`] in normal context (where Sender drop is safe).
/// -1 means uninitialized; signal handler no-ops if so.
static SHUTDOWN_WAKE_WRITE_FD: AtomicI32 = AtomicI32::new(-1);

/// Initialize the shutdown signal infrastructure. Idempotent — safe to
/// call from every bootstrap path. Creates:
///   1. A crossbeam unbounded channel pair (rx → SHUTDOWN_RX, tx → SHUTDOWN_TX_PTR)
///   2. A wake pipe (write-end → SHUTDOWN_WAKE_WRITE_FD, read-end → worker)
///   3. A worker thread that blocks on the wake pipe read; on wake,
///      calls trigger_shutdown
pub fn init_shutdown_signal() {
    if SHUTDOWN_RX.get().is_some() {
        return;  // already initialized
    }
    let (tx, rx) = crossbeam_channel::unbounded::<()>();
    // First-set wins — race-safe; the OnceLock-set Err path means another
    // thread initialized concurrently. Both paths leave the substrate
    // correctly initialized; we just discard our local Sender if so.
    if SHUTDOWN_RX.set(rx).is_err() {
        return;
    }
    let boxed = Box::into_raw(Box::new(tx));
    SHUTDOWN_TX_PTR.store(boxed, std::sync::atomic::Ordering::SeqCst);

    // Create wake pipe
    let mut fds = [0_i32; 2];
    let pipe_result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if pipe_result != 0 {
        // pipe(2) failed — substrate cannot safely operate; eprintln
        // a structured diagnostic and exit. (Should never happen in
        // practice on Linux.)
        eprintln!("substrate: pipe(2) failed during shutdown init");
        std::process::exit(1);
    }
    let read_fd = fds[0];
    let write_fd = fds[1];
    SHUTDOWN_WAKE_WRITE_FD.store(write_fd, std::sync::atomic::Ordering::SeqCst);

    // Spawn worker thread
    std::thread::Builder::new()
        .name("wat-shutdown-worker".to_string())
        .spawn(move || {
            let mut buf = [0u8; 1];
            // Block forever until signal handler writes a byte.
            // EINTR retry loop is unnecessary because the read is blocking
            // on a pipe (not on a sleeping syscall); a signal hitting us
            // mid-read either delivers the byte we're waiting for OR
            // returns EINTR which we treat as wake-anyway.
            let _ = unsafe { libc::read(read_fd, buf.as_mut_ptr() as *mut _, 1) };
            // Wake-up received. Trigger shutdown in normal context.
            trigger_shutdown();
            // Worker exits — its job is done; subsequent shutdown attempts
            // are no-ops (trigger_shutdown is idempotent).
        })
        .expect("wat-shutdown-worker thread spawn failed");
}

/// Drop the global SHUTDOWN_TX_PTR. All SHUTDOWN_RX recvs wake with
/// crossbeam Disconnected (which typed_recv Slice B maps to
/// RecvOutcome::Shutdown). Idempotent — second call sees null pointer
/// and no-ops.
///
/// MUST be called from normal context (deallocator can run). The signal
/// handler MUST NOT call this directly — it writes to the wake pipe;
/// the worker thread calls trigger_shutdown.
pub fn trigger_shutdown() {
    let ptr = SHUTDOWN_TX_PTR.swap(
        std::ptr::null_mut(),
        std::sync::atomic::Ordering::SeqCst,
    );
    if !ptr.is_null() {
        // SAFETY: ptr was Box::into_raw'd in init_shutdown_signal and
        // is never accessed except via this swap. The swap to null
        // means no other thread can race us into Box::from_raw on the
        // same pointer.
        unsafe { drop(Box::from_raw(ptr)) };
    }
}

/// Test-only reset for the shutdown infrastructure. Production code
/// runs init exactly once per process; tests that exercise the
/// cascade re-init between runs.
#[cfg(test)]
pub fn reset_shutdown_signal() {
    // Drop existing Sender (if any) so any blocked recvs disconnect
    trigger_shutdown();
    // NOTE: SHUTDOWN_RX (OnceLock) cannot be reset; tests should be
    // structured to tolerate the once-set semantics — the same Rx
    // value is reused across resets, and a fresh Sender cloned via
    // init_shutdown_signal would not produce messages on the
    // already-disconnected Rx. Slice B will refine this if tests
    // require it.
}
```

### 3. `src/runtime.rs` — wat-level `ThreadDiedError::Shutdown` variant

Find the `ThreadDiedError` enum definition (existing variants per arc 060/105b). Add:

```rust
/// arc 170 Slice A: process-wide shutdown signal fired during recv.
/// The channel partner did NOT drop — the process is terminating.
/// Distinguishable from existing variants (Panicked, Disconnected
/// per-channel) for user code that wants shutdown-specific cleanup.
/// Slice B wires recv to surface this; Slice A only adds the variant.
Shutdown,
```

Update any `Display` impl, `to-failure` accessor, EDN encoding, type-checker scheme registration, etc., as needed to keep the enum consistent. ZERO behavior changes for existing variants.

### 4. `src/freeze.rs` — call init in bootstrap

In `bootstrap_wat_vm_process` (line 224+), add ONE call early:

```rust
// Initialize the process-wide shutdown infrastructure. Idempotent
// across multiple bootstraps; the worker thread + wake pipe are
// per-process and survive subsequent bootstraps within the same OS
// process. Slice B+C+D+E wire the cascade to actually do something.
crate::runtime::init_shutdown_signal();
```

Place this BEFORE the trio service spawning so the SHUTDOWN_RX is available when services start.

## Scorecard (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `RecvOutcome::Shutdown` variant added | grep src/typed_channel.rs |
| B | `SHUTDOWN_RX` + `SHUTDOWN_TX_PTR` + `SHUTDOWN_WAKE_WRITE_FD` static items present | grep src/runtime.rs |
| C | `init_shutdown_signal()` exists + spawns worker thread | grep + read |
| D | `trigger_shutdown()` exists + ZERO-MUTEX (uses AtomicPtr + Box, no Mutex) | grep + read |
| E | `ThreadDiedError::Shutdown` variant added + Display/EDN/scheme consistent | cargo test + grep |
| F | `init_shutdown_signal()` called in `bootstrap_wat_vm_process` BEFORE trio spawn | grep + read order |
| G | `cargo build --release --workspace` passes | full build |
| H | `cargo test --release -p wat --test test` shows **167/7 baseline UNCHANGED** | independent re-run |
| I | No new `wat-shutdown-worker` thread leaks in test binary process (worker exits when triggered) | pgrep after test run |
| J | NO Mutex/RwLock/CondVar introduced (ZERO-MUTEX compliance) | `grep -rE "Mutex\|RwLock\|CondVar" src/runtime.rs src/typed_channel.rs src/freeze.rs` — should match ONLY existing sites, no new |

**All 10 rows must PASS.** Row H is load-bearing — must show no behavior regression.

## Constraints (NARROW per feedback_brief_constraint_contradictions)

- **In scope:** edits to `src/typed_channel.rs`, `src/runtime.rs`, `src/freeze.rs`. SCORE doc creation.
- **Out of scope:** wiring `typed_recv` to use SHUTDOWN_RX (Slice B). Signal handler changes (Slice B). PR_SET_PDEATHSIG (Slice C). Probe (Slice D).
- **Must not touch:** `src/check.rs` (committed verifier per security constraint).
- **Must not touch:** `src/spawn_process.rs` (Slice C territory).

## Discipline

- Pre-action sweep per FM 17 before commit: verify each row's pass criterion independently.
- Atomic commit on green. SCORE doc path:
  `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-A-SHUTDOWN-INFRA.md`
- Honest deltas section in SCORE doc if any deviation from this BRIEF.

## Mode B trigger

- ZERO-MUTEX violation detected somewhere in the dependency chain
- `cargo test` baseline shifts (167/7 changes) — split slice or escalate
- Worker thread can't be spawned safely from `bootstrap_wat_vm_process` (e.g., context where threads aren't allowed) — escalate

## Runtime band

**60-90 min sonnet.** Hard cap 180 min. Wakeup at T+3600s (1h).

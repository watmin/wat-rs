# Arc 214 Slice 3 — Stone B — Cascade-aware multi-arm POLL_ADD

## Mission

Stone B is the SECOND of five stepping stones in Slice 3. It proves **one** thing: io_uring multi-arm POLL_ADD on `[data_fd, broadcast_fd]` correctly wakes blocked recvs on substrate shutdown, with broadcast winning ties.

This stone wires the cascade contract that Stone A's module-level doc named as "NOT WIRED IN STONE A":

> ## Cascade contract (NOT WIRED IN STONE A)
> Stone B wires `SHUTDOWN_BROADCAST_READ_FD` as a second POLL_ADD arm so that substrate shutdown wakes blocked recvs.

After Stone B, the process tier matches Slice 2's thread-tier cascade discipline: blocked recvs cannot hang past substrate shutdown.

Stone B still defers:
- Generic `T: HolonRepresentable` serialization (Stone C)
- `try_recv` + `Select` + Clone + close + len + trait impls (Stone D)
- Persistent IoUring + config tunable (Stone E)

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone B)

- **Stone A (shipped):** io_uring bytes-only `pair()` + `Sender::send(&[u8])` + `Receiver::recv()` with newline framing. Per-call IoUring. NO cascade.
- **Stone B (this work):** cascade-aware multi-arm POLL_ADD on `[data_fd, broadcast_fd]`. Broadcast wins ties. Bootstrap fallback when `SHUTDOWN_BROADCAST_READ_FD == -1`.
- **Stone C:** Make Sender/Receiver generic over `T: HolonRepresentable`; HolonAST ↔ EDN bytes via wat-edn.
- **Stone D:** `try_recv` + `Select<'a, T>` + Clone + close + len + CommSender/CommReceiver trait impls.
- **Stone E:** Persistent IoUring per Receiver (via interior mutability) + `:wat::config::set-process-tier-uring-depth!`.

## Substrate context (substrate-truth verified pre-spawn)

- **`SHUTDOWN_BROADCAST_READ_FD: AtomicI32`** at `src/runtime.rs:201` — initialized to `-1`; valid fd after `init_shutdown_signal_with_inputs` runs (idempotent). Substrate-worker holds the write-end; drops it after `trigger_shutdown()`. Read-side sees `POLLHUP` when the write-end is gone.
- **Pattern reference (READ-ONLY):** `src/typed_channel.rs:329-368` — existing `libc::poll`-based PipeFd recv uses `pollfd` array with:
  - data fd: `libc::POLLIN | libc::POLLHUP`
  - broadcast fd: `libc::POLLHUP`
  - "Shutdown wins ties per Slice B discipline — process is going down; honest reporting" (lines 360-364)
  - Loop with `EINTR` retry
  Stone B mirrors this event-mask + tiebreak discipline via io_uring opcodes.
- **Stone A's existing code** at `src/comms/process.rs`:
  - `Receiver::recv` currently does fast-path accumulator check + per-call `IoUring::new(2)` + `opcode::Read` + EOF detection.
  - Stone B PREPENDS a cascade-aware wait step (multi-arm POLL_ADD); the Read step is unchanged.
- **io-uring 0.7 API:** `opcode::PollAdd::new(types::Fd(fd), poll_mask_u32).build().user_data(u64)` builds a POLL_ADD SQE. Per-call IoUring naturally handles un-fired arm cleanup at Drop (Stone A pattern).
- **Per-call IoUring** is intentional for Stones A/B; Stone E promotes to persistent IoUring. Stone B does NOT optimize.

## Concrete deliverables

### 1. Edit `src/comms/process.rs` — module-level doc

Update the "Cascade contract (NOT WIRED IN STONE A)" section to "Cascade contract (WIRED IN STONE B)". Specifically: replace the existing doc paragraph (around lines 36-44 in the current file; the section starts with "## Cascade contract (NOT WIRED IN STONE A)") with:

```rust
//! ## Cascade contract (Stone B)
//!
//! `Receiver::recv` is cascade-aware: every blocking recv polls both the
//! data fd and the substrate's `SHUTDOWN_BROADCAST_READ_FD` via io_uring
//! multi-arm `POLL_ADD`. Broadcast wins ties (the process is going down;
//! honest reporting). On shutdown, blocked recvs return `Err(RecvError)`
//! rather than hanging.
//!
//! Event masks match the substrate's existing PipeFd convention
//! (typed_channel.rs:329-368):
//!   - data fd: `POLLIN | POLLHUP` (data ready OR EOF)
//!   - broadcast fd: `POLLHUP` (worker dropped write-end on shutdown)
//!
//! Bootstrap fallback: when `SHUTDOWN_BROADCAST_READ_FD == -1` (pre-init
//! or test bypass), the cascade-poll step is skipped and recv falls back
//! to bare io_uring Read — same behavior as Stone A. Production paths
//! always have the broadcast pipe initialized before user code runs.
```

### 2. Edit `src/comms/process.rs` — add `PollOutcome` enum + `wait_for_data_or_cascade` helper

Add these two items immediately above the `take_frame` helper (i.e., they're module-private utilities used by `Receiver::recv`):

```rust
/// Outcome of a cascade-aware multi-arm wait. Internal to the
/// process-tier recv loop.
enum PollOutcome {
    /// Data fd's POLL_ADD fired (POLLIN or POLLHUP for EOF).
    /// Caller follows with an io_uring Read on the data fd.
    DataReady,
    /// Broadcast fd's POLL_ADD fired (POLLHUP — worker dropped
    /// the write-end on substrate shutdown). Caller returns
    /// `Err(RecvError)`.
    Shutdown,
}

/// Wait for either data readiness or substrate shutdown via io_uring
/// multi-arm `POLL_ADD`. Returns when at least one arm fires; both
/// arms may fire simultaneously, in which case broadcast wins
/// (substrate-shutdown takes precedence over pending data).
///
/// Per-call `IoUring::new(4)` — Stone B uses 4 entries to hold
/// 2 POLL_ADD SQEs plus headroom (Stone E persistifies the ring).
/// Un-fired arms die with the ring at Drop; no explicit cancel
/// needed.
///
/// Event masks match `src/typed_channel.rs:329-368` discipline:
///   - data fd: POLLIN | POLLHUP (data ready OR peer-closed)
///   - broadcast fd: POLLHUP (worker dropped write-end)
///
/// Returns `Err(RecvError)` on io_uring submission/wait failure or
/// on a CQE error (`cqe.result() < 0`).
fn wait_for_data_or_cascade(
    read_fd: std::os::fd::RawFd,
    broadcast_fd: std::os::fd::RawFd,
) -> Result<PollOutcome, RecvError> {
    const DATA_TOKEN: u64 = 1;
    const BROAD_TOKEN: u64 = 2;

    let mut ring = IoUring::new(4).map_err(|_| RecvError)?;

    let poll_data = opcode::PollAdd::new(
        types::Fd(read_fd),
        (libc::POLLIN | libc::POLLHUP) as u32,
    )
    .build()
    .user_data(DATA_TOKEN);

    let poll_broad = opcode::PollAdd::new(
        types::Fd(broadcast_fd),
        libc::POLLHUP as u32,
    )
    .build()
    .user_data(BROAD_TOKEN);

    // SAFETY: both SQEs reference fds owned elsewhere
    // (read_fd by the Receiver; broadcast_fd by the substrate worker).
    // Both remain valid for the lifetime of this submit_and_wait call.
    unsafe {
        ring.submission()
            .push(&poll_data)
            .map_err(|_| RecvError)?;
        ring.submission()
            .push(&poll_broad)
            .map_err(|_| RecvError)?;
    }

    ring.submit_and_wait(1).map_err(|_| RecvError)?;

    // Drain ALL ready CQEs — both arms may fire simultaneously.
    let mut got_data = false;
    let mut got_broadcast = false;
    while let Some(cqe) = ring.completion().next() {
        if cqe.result() < 0 {
            return Err(RecvError);
        }
        match cqe.user_data() {
            DATA_TOKEN => got_data = true,
            BROAD_TOKEN => got_broadcast = true,
            // Unreachable: we only push two SQEs with these two tokens.
            _ => return Err(RecvError),
        }
    }

    // Broadcast wins ties — substrate is going down; honest reporting
    // (mirrors typed_channel.rs:360-364 discipline).
    if got_broadcast {
        Ok(PollOutcome::Shutdown)
    } else if got_data {
        Ok(PollOutcome::DataReady)
    } else {
        // submit_and_wait(1) returned but no CQE drained — defensive.
        // Should not happen with min_complete=1; if it does, treat as
        // transient and let the caller retry via its loop.
        Err(RecvError)
    }
}
```

### 3. Edit `src/comms/process.rs` — `Receiver::recv` to use the cascade-aware pre-poll

Replace the current `Receiver::recv` body (the entire `loop { ... }` block; do NOT touch the fast-path accumulator check at the top). The new shape PREPENDS the cascade-aware wait step before the existing Read step:

```rust
pub fn recv(&self) -> Result<Vec<u8>, RecvError> {
    // Fast path — accumulator already has a complete frame.
    if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
        return Ok(frame);
    }

    let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
        .load(std::sync::atomic::Ordering::SeqCst);
    let read_fd = self.read_fd.as_raw_fd();

    loop {
        // Cascade-aware step — poll both arms (data + broadcast).
        // Bootstrap fallback: when broadcast_fd is -1 (pre-init or
        // test bypass), skip the poll and fall through to bare Read
        // (same behavior as Stone A; no cascade available).
        if broadcast_fd >= 0 {
            match wait_for_data_or_cascade(read_fd, broadcast_fd)? {
                PollOutcome::Shutdown => return Err(RecvError),
                PollOutcome::DataReady => {
                    // Data is ready; fall through to Read step.
                }
            }
        }

        // Read step — same as Stone A. Per-call IoUring; ring size 2.
        // (Stone E persistifies the ring.)
        let mut ring = IoUring::new(2).map_err(|_| RecvError)?;
        let mut buf = [0u8; 4096];
        let read_e = opcode::Read::new(
            types::Fd(read_fd),
            buf.as_mut_ptr(),
            buf.len() as _,
        )
        .build()
        .user_data(1);

        // SAFETY: read_e's buf pointer (buf) outlives submit_and_wait
        // because buf is on this function's stack and not freed until
        // after the wait completes.
        unsafe {
            ring.submission()
                .push(&read_e)
                .map_err(|_| RecvError)?;
        }

        ring.submit_and_wait(1).map_err(|_| RecvError)?;
        let cqe = ring.completion().next().ok_or(RecvError)?;
        let result = cqe.result();
        if result < 0 {
            return Err(RecvError);
        }
        if result == 0 {
            // EOF — peer closed the write-end.
            return Err(RecvError);
        }
        let n = result as usize;
        self.accumulator
            .borrow_mut()
            .extend_from_slice(&buf[..n]);

        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            return Ok(frame);
        }
        // No complete frame yet; loop and poll/read more bytes.
    }
}
```

### 4. NO new probe tests

All 6 Stone A probe tests already exercise the recv code path. Under Stone B, they exercise the cascade-aware variant (when `SHUTDOWN_BROADCAST_READ_FD` is initialized in the test environment) OR the bootstrap fallback variant (when it is not). Either way, the data path must work identically — the existing tests are the regression coverage for "cascade-arm wiring does not break data flow".

End-to-end cascade verification (broadcast actually fires; recv returns `Err(RecvError)`) is integration-level and lives in later slices (Slice 4/5). Per Slice 2 precedent: the cascade-aware pattern is verified STRUCTURALLY (the code reads correctly + wards confirm) at the unit-test level, not by triggering substrate shutdown in unit tests.

## Verification

```
cargo build --release                                       # MUST be clean (no new warnings)
cargo test --release --test probe_comms_process             # 6/6 PASS unchanged (cascade-aware data path)
cargo test --release --test probe_comms_thread              # 10/10 PASS unchanged
cargo test --release --test probe_comms_foundation          # 3/3 PASS unchanged (Slice 1)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged (χ-1)
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged (α)
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts` or any workspace tests.

## Out of scope (STOP triggers)

- **DO NOT make Sender/Receiver generic over T** — Stone C
- **DO NOT implement try_recv** — Stone D
- **DO NOT implement Select** — Stone D
- **DO NOT implement Clone** — Stone D
- **DO NOT implement close(self)** — Stone D
- **DO NOT implement len()** — Stone D
- **DO NOT implement CommSender / CommReceiver trait impls** — Stone D
- **DO NOT add config tunable** — Stone E
- **DO NOT optimize to persistent IoUring** — Stone E
- **DO NOT cancel un-fired POLL_ADDs explicitly** — per-call IoUring drops them at end of `wait_for_data_or_cascade`; explicit cancel is unnecessary overhead at this stone
- **DO NOT add new probe tests** — 6 inherited Stone A tests are sufficient regression coverage; end-to-end cascade is integration-level
- **DO NOT touch the dirty tree** (`src/fork.rs` + `src/spawn_process.rs`)
- **DO NOT touch `src/typed_channel.rs`** (existing PipeFd; Slice 5 migrates callers later)
- **DO NOT run `wat_arc170_program_contracts`** (per additive-scorecard discipline)
- **ZERO modifications** outside `src/comms/process.rs` (Sender + take_frame + pair UNCHANGED; only module doc + Receiver::recv body + 2 new private items)

## Pre-emptive ward discipline (lessons from Slices 1 + 2 + Stone A)

1. **All public items keep their doc comments** — Stone B does not add public items; the existing `pub fn recv` doc updates to reflect cascade-aware status.
2. **PRIVATE items get doc comments too** (`wait_for_data_or_cascade`, `PollOutcome`) — explains intent + the broadcast-wins-ties discipline + the bootstrap-fallback condition.
3. **Comments explain WHY not WHAT** — the "broadcast wins ties" comment explains the SUBSTRATE INVARIANT (process is going down; honest reporting) not the mechanical fact (we check `got_broadcast` first).
4. **SAFETY comment at every unsafe block** (Stone A round-1 forge lesson) — the new `unsafe { ... }` block in `wait_for_data_or_cascade` has a SAFETY comment naming the fd-ownership-elsewhere + lifetime invariant.
5. **Event masks match existing PipeFd convention** — `POLLIN | POLLHUP` for data, `POLLHUP` for broadcast. Match the substrate's existing convention; don't re-litigate masks.
6. **The Stone-A `Receiver::recv` code body is partially preserved**, NOT rewritten from scratch — the Read step is verbatim (mod removing one trailing comment). The accumulator + take_frame logic is unchanged.

## Concrete deliverables list

1. **Edit** `src/comms/process.rs` — module-level doc (replace "NOT WIRED IN STONE A" section with "Cascade contract (Stone B)" section)
2. **Edit** `src/comms/process.rs` — add `PollOutcome` enum + `wait_for_data_or_cascade` helper above the `take_frame` helper
3. **Edit** `src/comms/process.rs` — `Receiver::recv` body uses the new helper as a cascade-aware pre-poll
4. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md`
5. **NO new probe tests** — existing 6 tests are regression coverage

Estimated LOC: ~70-90 LOC added to `src/comms/process.rs` (PollOutcome + wait_for_data_or_cascade + Receiver::recv refactor); module doc edit + ~10 lines updated.

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + 5-ward pass per kernel-impeccability protocol.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path.
- **Use `git -C`** for any git status / git diff inspections.

## Cross-references

- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — Stone A work order (FOUNDATION)
- WARD-PASS-3A-IO-URING-BYTES.md — Stone A 5-ward round-trip (lessons inherited)
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design; Slice 3 cascade contract
- `src/typed_channel.rs:329-368` — existing libc::poll cascade pattern (READ-ONLY reference; mirror its event-mask + tiebreak discipline)
- `src/runtime.rs:201` — `SHUTDOWN_BROADCAST_READ_FD` definition + init mechanism
- `src/comms/process.rs` — Stone A's current state (Sender + Receiver + take_frame + pair)
- `src/comms/thread.rs` — Slice 2 thread-tier cascade-aware recv reference (different mechanism, same contract shape)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — per-stone trust gate protocol
- `feedback_no_hang_vector_in_additive_scorecard` — verification discipline
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — why 5 stones in Slice 3

# Arc 214 Slice 3 — Stone E-1 — Receiver persistent ring (capacity 4)

## Mission

Stone E-1 is the SIXTH of seven stepping stones in Slice 3. It proves **one** thing: the Receiver owns a persistent `IoUring` (capacity 4) for the lifetime of the Receiver, replacing the per-call `IoUring::new(2)` and `IoUring::new(4)` constructions in `uring_read_into_acc` + `wait_for_data_or_cascade`.

Per DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild": Receiver's operation set is STATIC for the Receiver's lifetime (recv + try_recv + Read-on-behalf-of-Select use the same ring shape — Read OR POLL_ADD-pair; max 2 SQEs at any moment; capacity 4 covers both with headroom). No reconstruction in E-1; reflexive rebuild lives in E-2 (Select's variable arm_count).

This stone is mechanical: lift the IoUring constructions OUT of the helpers, store them in the Receiver, refactor helpers to take `&IoUring` (or become `&self` methods).

Migrates 2 Receiver runes from `rune:temperare(no-reactor)` to cold: the helpers no longer construct per-call rings.

After E-1, the Receiver's ring is the persistent kernel resource; FDs persist alongside it; both drop at Receiver Drop.

E-1 still defers:
- Select's persistent ring + reflexive rebuild-on-mismatch (Stone E-2)

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone E-1)

- **Stone A (shipped):** io_uring bytes-only foundation
- **Stone B (shipped):** cascade-aware 2-arm POLL_ADD
- **Stone C (shipped):** generic `T: HolonRepresentable` + serialization
- **Stone D1 (shipped):** mechanical methods (try_recv + len + close + Clone + traits)
- **Stone D2 (shipped):** `Select<'a, T>` cascade-aware N+1-arm fan-in
- **Stone E-1 (this work):** Receiver persistent ring (capacity 4; static-need)
- **Stone E-2:** Select persistent ring with reflexive rebuild-on-mismatch

## Substrate context (substrate-truth verified pre-spawn)

- **`src/comms/process.rs:204-217`** — current `Receiver<T>` struct: `read_fd: OwnedFd`, `accumulator: RefCell<Vec<u8>>`, `_phantom: PhantomData<T>`. E-1 adds a fourth field: `ring: RefCell<IoUring>`.
- **`src/comms/process.rs:384-406`** — current `Receiver::clone` impl: `try_clone` the fd; FRESH empty accumulator; new PhantomData. E-1 amends Clone to ALSO mint a fresh `IoUring::new(4)` per clone (rings are not Sync; each clone has its own).
- **`src/comms/process.rs:451-516`** — current `wait_for_data_or_cascade(read_fd, broadcast_fd)` free function: per-call `IoUring::new(4)`; submits POLL_ADDs on data + broadcast; drains CQEs; broadcast wins ties. E-1 refactors to take `&IoUring` ring parameter (caller supplies the Receiver's ring).
- **`src/comms/process.rs:569-596`** — current `uring_read_into_acc(fd, acc)` free function: per-call `IoUring::new(2)`; submits Read SQE; drains one CQE. E-1 refactors to take `&IoUring` ring parameter.
- **`src/comms/process.rs:812-` `pair<T>()`** — current factory: creates pipe via `libc::pipe(2)`; wraps fds as Sender + Receiver; constructs Receiver with empty accumulator. E-1 amends to also construct `IoUring::new(4)` for the Receiver's ring.
- **`Select::select` at `src/comms/process.rs:664-` Read step (line 778):** currently calls `uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator)`. E-1 amends to pass `&rx.ring` as a third arg so Select uses the fired Receiver's persisted ring for the Read step (NOT Select's own ring — Select gets its own ring in E-2). The Select still uses a PER-CALL ring for the POLL_ADD step (Stone E-2 territory).

**Critical: io_uring is `Send` but NOT `Sync`.** Each Receiver owns its ring; clones get fresh rings. This is honest because:
1. Receiver is `!Sync` already (RefCell<Vec<u8>>); the !Sync property propagates
2. The substrate's threading model never shares a Receiver across threads; clones are how cross-thread fan-out works
3. Stone E-2 will mirror this for Select

**Ring reuse semantics:** io_uring rings are reusable across submit-wait-drain cycles. After `submit_and_wait(N)`, you can drain CQEs, then push new SQEs, then submit again. The ring's submission queue + completion queue are the persistent kernel resources; the SQEs/CQEs flow through them.

## Concrete deliverables

### 1. Update `src/comms/process.rs` module-level doc

Replace `## Current scope (through Stone D2)` section's last sentence with one that names the E-1 capability:

```rust
//! ## Current scope (through Stone E-1)
//!
//! Full API surface matching the thread tier (`crate::comms::thread`).
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware multi-arm
//! POLL_ADD (Stone B). io_uring bytes foundation with newline framing
//! (Stone A). Stone D1: try_recv + len + close + Clone + CommSender/
//! CommReceiver trait impls. Stone D2: `Select<'a, T>` — cascade-aware
//! fan-in over N receivers (generalizes Stone B's 2-arm POLL_ADD to
//! N+1 arms; broadcast wins ties). Stone E-1: Receiver owns persistent
//! IoUring (capacity 4) for its lifetime; helpers operate on the
//! Receiver's ring instead of per-call construction. Only NO persistent
//! ring on Select / reflexive rebuild-on-mismatch (Stone E-2).
```

### 2. Add `ring` field to `Receiver<T>` struct

Update the struct definition at `src/comms/process.rs:204-217`:

```rust
/// Receive process-tier values. Wraps the read-end of an
/// `OwnedFd` pipe; decodes newline-framed EDN payloads to `T`.
/// `Clone` competes for frames via `try_clone` (Stone D1);
/// each clone gets a FRESH empty accumulator AND a fresh ring
/// (rings are `Send` but `!Sync`; never share across clones).
/// Stone E-1: ring is persistent for the Receiver's lifetime;
/// capacity 4 covers Read (1 SQE) and POLL_ADD pair (2 SQEs)
/// operations with headroom.
#[derive(Debug)]
pub struct Receiver<T: HolonRepresentable> {
    read_fd: OwnedFd,
    /// Bytes read from the pipe but not yet returned to a caller.
    /// `RefCell` provides interior mutability so `recv(&self)` can
    /// update the accumulator without `&mut self`. `Receiver` is `!Sync`
    /// by construction (RefCell is !Sync); the substrate's threading
    /// model never shares a single Receiver across threads — clones
    /// (Stone D) create independent endpoints.
    accumulator: RefCell<Vec<u8>>,
    /// Persistent io_uring (Stone E-1) — capacity 4 covers Read
    /// (1 SQE) and POLL_ADD pair (2 SQEs) operations with headroom.
    /// `RefCell` for the same `&self` interior-mutability reason as
    /// the accumulator. Constructed at `pair()` and at `Clone`; dropped
    /// at Receiver Drop (kernel resource cleaned up via IoUring's own
    /// Drop impl).
    ring: RefCell<IoUring>,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `recv` produces. `PhantomData<T>` makes `Receiver<T>`
    /// invariant in T which is correct for this use case.
    _phantom: PhantomData<T>,
}
```

### 3. Refactor `uring_read_into_acc` to take a ring parameter

Change the free function signature and body at `src/comms/process.rs:559-596`:

```rust
/// Issues one io_uring Read on `fd` into `acc` using the supplied
/// persistent ring `ring`. Returns `Ok(n)` where `n` is the number
/// of bytes appended (0 means EOF / peer closed write end), or
/// `Err(())` on SQE submission, submit_and_wait, or CQE error.
///
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver (or, in Select's Read-step, from the fired
/// Receiver). Per-call `IoUring::new(2)` is retired.
///
/// Callers map `Err(())` to their domain error (RecvError, TryRecvError,
/// etc.) at the call site.
fn uring_read_into_acc(
    fd: std::os::fd::RawFd,
    acc: &std::cell::RefCell<Vec<u8>>,
    ring: &std::cell::RefCell<IoUring>,
) -> Result<usize, ()> {
    let mut ring = ring.borrow_mut();
    let mut buf = [0u8; 4096];
    let read_e = opcode::Read::new(
        types::Fd(fd),
        buf.as_mut_ptr(),
        buf.len() as _,
    )
    .build()
    .user_data(1);

    // SAFETY: read_e's buf pointer (buf) outlives submit_and_wait because
    // buf is on this function's stack and is not freed until after the wait
    // completes.
    unsafe {
        ring.submission().push(&read_e).map_err(|_| ())?;
    }

    ring.submit_and_wait(1).map_err(|_| ())?;
    let cqe = ring.completion().next().ok_or(())?;
    let result = cqe.result();
    if result < 0 {
        return Err(());
    }
    let n = result as usize;
    acc.borrow_mut().extend_from_slice(&buf[..n]);
    Ok(n)
}
```

**Rune deletion:** the `rune:temperare(no-reactor)` doc-comment on this helper is REMOVED (cold; no longer per-call construction).

### 4. Refactor `wait_for_data_or_cascade` to take a ring parameter

Change the free function signature and body at `src/comms/process.rs:435-516`:

```rust
/// Wait for either data readiness or substrate shutdown via io_uring
/// multi-arm `POLL_ADD`. Returns when at least one arm fires; both
/// arms may fire simultaneously, in which case broadcast wins
/// (substrate-shutdown takes precedence over pending data).
///
/// Stone E-1: ring is now a persistent kernel resource borrowed from
/// the calling Receiver. Per-call `IoUring::new(4)` is retired.
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
    ring: &std::cell::RefCell<IoUring>,
) -> Result<PollOutcome, RecvError> {
    const DATA_TOKEN: u64 = 1;
    const BROADCAST_TOKEN: u64 = 2;

    let mut ring = ring.borrow_mut();

    let poll_data = opcode::PollAdd::new(
        types::Fd(read_fd),
        (libc::POLLIN | libc::POLLHUP) as u32,
    )
    .build()
    .user_data(DATA_TOKEN);

    let poll_broadcast = opcode::PollAdd::new(
        types::Fd(broadcast_fd),
        libc::POLLHUP as u32,
    )
    .build()
    .user_data(BROADCAST_TOKEN);

    // SAFETY: both SQEs reference fds owned elsewhere
    // (read_fd by the Receiver; broadcast_fd by the substrate worker).
    // Both remain valid for the lifetime of this submit_and_wait call.
    unsafe {
        ring.submission()
            .push(&poll_data)
            .map_err(|_| RecvError)?;
        ring.submission()
            .push(&poll_broadcast)
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
            BROADCAST_TOKEN => got_broadcast = true,
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

**Rune deletion:** the `rune:temperare(no-reactor)` doc-comment on this helper is REMOVED (cold; no longer per-call construction).

### 5. Update `Receiver::recv` call sites

At `src/comms/process.rs:248` and `src/comms/process.rs:259`, pass `&self.ring`:

```rust
// Line ~248 (inside Receiver::recv loop):
match wait_for_data_or_cascade(read_fd, broadcast_fd, &self.ring)? {
    PollOutcome::Shutdown => return Err(RecvError),
    PollOutcome::DataReady => {
        // Data is ready; fall through to Read step.
    }
}

// ...

// Line ~259 (Read step in Receiver::recv):
let n = uring_read_into_acc(read_fd, &self.accumulator, &self.ring).map_err(|_| RecvError)?;
```

Doc-comment rune references near these sites (lines ~256-258, 338) are REMOVED.

### 6. Update `Receiver::try_recv` call site

At `src/comms/process.rs:351` (inside `try_recv`), pass `&self.ring`:

```rust
// Inside try_recv, the Read step:
let n = uring_read_into_acc(read_fd, &self.accumulator, &self.ring).map_err(|_| TryRecvError::Disconnected)?;
```

(Verify the exact line by reading `try_recv`'s current body; the change is purely the extra argument.)

### 7. Update `Receiver::clone` to construct a fresh ring

At `src/comms/process.rs:384-406`:

```rust
impl<T: HolonRepresentable> Clone for Receiver<T> {
    /// Clone the receiver by duplicating its read-end fd via
    /// `OwnedFd::try_clone`. Both clones reference the same kernel
    /// pipe and COMPETE for frames — a frame consumed by one clone
    /// is gone from the pipe (MPMC-style read fan-out).
    ///
    /// The cloned receiver gets a FRESH empty accumulator — it does
    /// NOT inherit the original's buffered bytes. Accumulator state
    /// is per-endpoint; sharing it would create confusing partial-frame
    /// behavior across clones.
    ///
    /// Stone E-1: the cloned receiver also gets a FRESH IoUring
    /// (capacity 4) — rings are `Send` but `!Sync`; each clone owns
    /// its own ring so clones operating on different threads do not
    /// race on the ring's submission/completion queues.
    ///
    /// Panics on `libc::dup` failure (EMFILE/ENFILE; fd table exhausted)
    /// or `IoUring::new(4)` failure (kernel resource exhaustion; rare).
    fn clone(&self) -> Self {
        Self {
            read_fd: self
                .read_fd
                .try_clone()
                .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            accumulator: RefCell::new(Vec::new()),
            ring: RefCell::new(
                IoUring::new(4)
                    .expect("IoUring::new(4) failed — kernel io_uring resource exhausted"),
            ),
            _phantom: PhantomData,
        }
    }
}
```

### 8. Update `pair<T>()` factory to construct the Receiver's ring

At `src/comms/process.rs:812-` (the `pair<T>()` body that constructs the Receiver):

```rust
let receiver = Receiver {
    read_fd: read_owned,
    accumulator: RefCell::new(Vec::new()),
    ring: RefCell::new(
        IoUring::new(4)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
                format!("IoUring::new(4) failed at Receiver construction: {}", e)))?,
    ),
    _phantom: PhantomData,
};
```

(Exact spelling of the surrounding factory may differ; the change is: add the `ring:` field with `IoUring::new(4)` wrapped in RefCell. Failure surfaces as the existing `std::io::Result` return type.)

### 9. Update `Select::select`'s Read step

At `src/comms/process.rs:778`, change the call from:

```rust
match uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator) {
```

to:

```rust
match uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator, &rx.ring) {
```

Select borrows the FIRED Receiver's ring for the Read step. Select's own POLL_ADD ring stays per-call (Stone E-2 persistifies it).

### 10. Tests — preserve all 34 existing tests; verify they still pass

Stone E-1 is mechanically invisible to test surface. NO new tests. ALL 34 existing tests must pass unchanged:

- `tests/comms/foundation.rs` — preserved
- `tests/comms/thread.rs` — preserved
- `tests/comms/process.rs` — preserved (all `probe_slice3c_*` + `probe_slice3d1_*` + `probe_slice3d2_*` tests)

## Verification

```
cargo build --release                                       # MUST be clean
cargo test --release --test comms                           # 34/34 PASS (zero net delta from Phase 2)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts`.

## Out of scope (STOP triggers)

- **DO NOT touch `Select<'a, T>`'s ring** — Stone E-2 persistifies it with reflexive rebuild
- **DO NOT add config tunable** — DISQUALIFIED by four-questions per DESIGN.md § "Stone E forward-correction (2026-05-19)"
- **DO NOT add new probe tests** — E-1 is mechanically invisible to test surface; 34/34 unchanged
- **DO NOT modify** Stone A's `take_frame`, Stone B's `PollOutcome` enum, Stone C's `decode_frame` / `Sender::send` / `pair<T>()` Sender-side, Stone D1's methods + trait impls (Receiver close/len/CommReceiver), Stone D2's `Select::new` / `Select::recv` / `Select::select`'s POLL_ADD step (only the Read step at line 778 changes)
- **DO NOT touch the dirty tree** — `src/fork.rs` + `src/spawn_process.rs` are arc 213 territory
- **DO NOT touch `src/typed_channel.rs`**, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml`
- **DO NOT run** `wat_arc170_program_contracts`
- **ZERO modifications** outside the 1-file scope (`src/comms/process.rs`) + SCORE doc

## Pre-emptive ward discipline (lessons from Stones A-D2 + Phase 1/2 cleanup)

1. **Module-level doc update** (Stone B gaze L1 lesson) — replace "Current scope (through Stone D2)" with "(through Stone E-1)" naming the new Receiver-persistent-ring capability.
2. **Doc comments on every modified item** (gaze pre-emption) — Receiver struct, ring field, refactored helpers all need doc-comments naming Stone E-1's contribution.
3. **`unsafe` blocks carry SAFETY comments** — refactored helpers retain their existing SAFETY comments; lifetimes still hold (ring borrow_mut() for the duration of the operation).
4. **Rune deletion** — the two `rune:temperare(no-reactor)` doc-comments on `uring_read_into_acc` and `wait_for_data_or_cascade` are REMOVED entirely. Per `feedback_inscription_immutable`: the rune is no longer applicable; deleting is honest. (The Select rune at line 688 remains until E-2.)
5. **RefCell discipline** — ring is `RefCell<IoUring>` for the same `&self` interior-mutability reason as accumulator. borrow_mut() takes the ring for the duration of one operation; releases at end of operation. Receiver is `!Sync` already; this propagates.
6. **NO panic-paths on the ring borrow** — RefCell borrow_mut() can panic on double-borrow. The substrate's threading model never shares a Receiver across threads (clones do); within a single Receiver, no recursive ring use exists (recv calls wait_for_data_or_cascade then uring_read_into_acc sequentially, releasing the borrow between).
7. **Clone honesty** — clones get FRESH rings (not Arc-shared); rings are !Sync; cloning the ring would create a use-after-free vector if both clones tried to submit on the same ring from different threads.
8. **Factory failure path** — `IoUring::new(4)` returning Err in `pair<T>()` surfaces as `std::io::Error::new(std::io::ErrorKind::Other, format!(...))` wrapping the underlying io error. Clone failure panics (rare kernel resource exhaustion).
9. **Select Read-step delegation** — Select uses the FIRED RECEIVER's ring for the Read step (rx.ring), not Select's own. Select's POLL_ADD ring stays per-call until E-2.
10. **NO new tests** — E-1 is mechanically invisible; existing tests prove correctness via behavior preservation.

## Concrete deliverables list

1. **Edit** `src/comms/process.rs` — module-level doc updated; `Receiver<T>` struct gains `ring: RefCell<IoUring>` field; `uring_read_into_acc` + `wait_for_data_or_cascade` refactored to take `&RefCell<IoUring>` parameter; `Receiver::recv` + `Receiver::try_recv` pass `&self.ring`; `Receiver::clone` constructs fresh `IoUring::new(4)`; `pair<T>()` constructs ring at Receiver creation; `Select::select`'s Read step passes `&rx.ring`; runes deleted from refactored helpers
2. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md`

Estimated LOC: ~30-60 LOC net delta (additions: ring field declaration + Clone ring construction + pair() ring construction + helper signature parameters; deletions: two `IoUring::new(...)` lines + rune comments). Mostly mechanical refactor; small net delta.

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + ward pass.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/`
- **Use `git -C`** for git ops

## Cross-references

- DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild" — the architectural reframe E-1 implements
- BRIEF-214-SLICE-3D2-SELECT.md — Stone D2 (last shipped pre-E-1)
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B (the 2-arm POLL_ADD pattern E-1 keeps; refactored to use Receiver's ring)
- SCORE-COMMS-CLEANUP-PHASE-1.md + SCORE-COMMS-CLEANUP-PHASE-2.md — vigilia cleanup that established the helper boundaries E-1 builds on
- `feedback_iterative_complexity` — Stone E split into E-1 + E-2 per four-questions (E-3 tunable rejected, not split)
- `feedback_options_are_tangle` — the discipline that rejected the tunable

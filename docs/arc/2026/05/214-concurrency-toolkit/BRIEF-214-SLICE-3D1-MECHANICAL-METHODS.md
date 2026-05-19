# Arc 214 Slice 3 — Stone D1 — Mechanical method additions + traits

## Mission

Stone D1 is the FOURTH of six stepping stones in Slice 3 (the original "Stone D" was split into D1 + D2 after the four-questions test disqualified the single-stone shape — bundling 6 distinct method/struct additions into one stone violated obvious/simple/honest/good-UX).

D1 proves **one** thing: the process tier's non-Select API surface methods (5 methods + 2 trait impls) match the thread tier's. After D1, `Sender<T>` and `Receiver<T>` carry `close`, `Clone`, `CommSender<T>` / `CommReceiver<T>` trait impls, and the Receiver also carries `try_recv` and `len`.

D1 explicitly defers Select<'a, T> to Stone D2 because Select is the only piece of Stone-D-original scope that's GENUINELY NEW substrate work (N+1-arm POLL_ADD generalized from Stone B's 2-arm pattern; per-call dual IoUring; bail-out semantics at N-arm scale). Isolating Select in its own stone keeps the per-stone trust gate's diagnostic value intact.

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone D1)

- **Stone A (shipped):** io_uring bytes-only `pair()` + `Sender::send(&[u8])` + `Receiver::recv()` with newline framing
- **Stone B (shipped):** cascade-aware multi-arm POLL_ADD on `[data_fd, broadcast_fd]`; broadcast wins ties
- **Stone C (shipped):** generic `Sender<T>` / `Receiver<T>` over `T: HolonRepresentable`; HolonAST ↔ EDN bytes; `impl HolonRepresentable for String`
- **Stone D1 (this work):** mechanical methods (try_recv + len + close + Clone) + traits (CommSender + CommReceiver)
- **Stone D2:** `Select<'a, T>` + Default impl (cascade-aware N+1-arm fan-in)
- **Stone E:** Persistent IoUring per Receiver + `:wat::config::set-process-tier-uring-depth!`

## Substrate context (substrate-truth verified pre-spawn)

- **`std::os::fd::OwnedFd::try_clone`** — `fn try_clone(&self) -> io::Result<OwnedFd>`. Internally calls `libc::dup` (or `dup3`/`F_DUPFD_CLOEXEC` on Linux). The returned fd references the same kernel file description; both clones share the pipe; the pipe stays alive while any fd to it exists. Fails only on EMFILE/ENFILE (fd table exhaustion).
- **`libc::poll(fds, nfds, timeout=0)`** — non-blocking arm check. Existing pattern at `src/typed_channel.rs:407-470` (typed_try_recv): poll [pipe_fd, broadcast_fd]; broadcast wins ties; on data → fall through to read.
- **Slice 1 trait surface at `src/comms/mod.rs:65-105`:**
  - `CommSender<T>` requires `send(&self, T) -> Result<(), SendError<T>>` + `close(self) -> Result<(), CloseError>`
  - `CommReceiver<T>` requires `recv(&self) -> Result<T, RecvError>` + `try_recv(&self) -> Result<T, TryRecvError>` + `len(&self) -> usize` + `close(self) -> Result<(), CloseError>`
  - `TryRecvError`: `Empty` (no data; may become ready) | `Disconnected` (peer dropped)
- **Slice 2 thread tier reference** at `src/comms/thread.rs` — process tier Sender/Receiver/CommSender/CommReceiver impls mirror this shape with libc::poll + io_uring underneath instead of crossbeam.
- **Stone B's `wait_for_data_or_cascade` + `PollOutcome`** — UNCHANGED by D1. Used by `recv` (blocking).
- **Stone C's `decode_frame::<T>` + `take_frame`** — UNCHANGED by D1. Used by `recv` + `try_recv`.
- **D1 leaves `Sender::send`, `Receiver::recv`, `pair<T>()` UNCHANGED.** All additions are appended.

## Concrete deliverables

### 1. Update `src/comms/process.rs` module-level doc

Replace `## Current scope (through Stone C)` section with:

```rust
//! ## Current scope (through Stone D1)
//!
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware
//! multi-arm POLL_ADD (Stone B). io_uring bytes foundation with
//! newline framing (Stone A). Stone D1 adds: `try_recv` (non-blocking
//! via libc::poll(timeout=0) + io_uring Read), `len` (accumulator
//! newline count; documented approximation), `close` (consume self;
//! OwnedFd Drop closes the fd), `Clone` (OwnedFd::try_clone duplicates
//! the fd; receivers compete MPMC-style for frames), CommSender/CommReceiver
//! trait impls. Still NO `Select<'a, T>` (Stone D2); NO persistent ring /
//! config tunable (Stone E).
```

### 2. Update Sender + Receiver struct docs

**Sender doc** — retire the "NOT Clone (Stone D adds Clone). NOT close-able (Stone D adds `close(self)`)" stale claims:

```rust
/// Process-tier send endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's write-end fd. Encodes `T` via
/// `HolonRepresentable::to_holon_ast` → `write_holon_ast_tagged` →
/// newline-framed bytes.
///
/// Clone via `OwnedFd::try_clone` (Stone D1); cloned senders share the
/// same kernel pipe (MPMC-style write fan-in). `close(self)` consumes
/// the endpoint and drops the fd via OwnedFd Drop; peer sees EOF after
/// ALL Sender clones close.
```

**Receiver doc** — retire the "NOT Clone (Stone D adds)" stale claim:

```rust
/// Process-tier receive endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's read-end fd and a small internal byte accumulator
/// for cross-call frame splitting.
///
/// Cascade-aware (Stone B): `recv` wakes on substrate shutdown via
/// io_uring multi-arm POLL_ADD on `SHUTDOWN_BROADCAST_READ_FD`. Stone D1
/// adds: `try_recv` (non-blocking variant via libc::poll(timeout=0));
/// `len` (accumulator-only frame count; kernel buffer not included);
/// `close` (consume self; OwnedFd Drop closes the fd); Clone via
/// `OwnedFd::try_clone` (cloned receivers compete for frames MPMC-style;
/// each clone gets a FRESH empty accumulator). Per-call `IoUring`
/// instance (Stone E persistifies).
```

### 3. Add imports

Update the existing import block in `src/comms/process.rs`:

```rust
use crate::comms::{
    CloseError, CommReceiver, CommSender, HolonRepresentable, RecvError,
    SendError, TryRecvError,
};
```

Adds: `CloseError`, `CommReceiver`, `CommSender`, `TryRecvError`. Stone D1 does NOT need `ReceiverIndex` or `SelectOutcome` (those are Stone D2's).

### 4. Add `Sender::close` + `Clone` + `CommSender<T>` impl

Append after the existing `Sender::send` method block:

```rust
impl<T: HolonRepresentable> Sender<T> {
    /// Signal end-of-stream from this sender. Consumes self so the
    /// endpoint is gone after close. Other cloned `Sender` handles
    /// (if any) remain valid. Peer receivers see EOF on their next
    /// recv only after ALL `Sender` clones close (the pipe's write
    /// reference count hits zero; kernel signals EOF on the read-end).
    ///
    /// Process-tier close always succeeds — OwnedFd Drop handles
    /// `libc::close(2)`; no fallible operation at this layer.
    pub fn close(self) -> Result<(), CloseError> {
        // self drops at end of scope; OwnedFd's Drop calls libc::close.
        // No fallible operation; always Ok.
        Ok(())
    }
}

impl<T: HolonRepresentable> Clone for Sender<T> {
    /// Clone the sender by duplicating its write-end fd via
    /// `OwnedFd::try_clone` (which uses `libc::dup` internally). Both
    /// clones reference the same kernel pipe; the pipe stays alive
    /// as long as at least one fd to it exists.
    ///
    /// Panics on `libc::dup` failure — this only happens at EMFILE/
    /// ENFILE (fd table exhaustion). A substrate-level fd-table
    /// exhaustion is a fail-stop condition; panicking with a
    /// diagnostic is honest reporting at this layer.
    fn clone(&self) -> Self {
        Self {
            write_fd: self
                .write_fd
                .try_clone()
                .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            _phantom: PhantomData,
        }
    }
}

impl<T: HolonRepresentable> CommSender<T> for Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        Sender::send(self, value)
    }
    fn close(self) -> Result<(), CloseError> {
        Sender::close(self)
    }
}
```

### 5. Add `Receiver::try_recv` + `Receiver::len` + `Receiver::close` + `Clone` + `CommReceiver<T>` impl

Append after the existing `Receiver::recv` method (inside the existing impl block OR in a new impl block — match the convention used in step 4):

```rust
impl<T: HolonRepresentable> Receiver<T> {
    /// Non-blocking recv. Returns the next complete `T` if a frame is
    /// already buffered (in the accumulator) OR if the data fd has bytes
    /// ready RIGHT NOW that complete a frame. Otherwise returns
    /// `Err(TryRecvError::Empty)`. Returns `Err(TryRecvError::Disconnected)`
    /// when the peer has closed the pipe (EOF) OR when substrate shutdown
    /// has fired (broadcast arm).
    ///
    /// Per substrate convention (typed_channel.rs:407-470): broadcast wins
    /// ties — shutdown overrides any pending Value (process going down;
    /// honest reporting).
    ///
    /// Mechanism: `libc::poll(timeout=0)` on `[data_fd, broadcast_fd]`
    /// for the non-blocking arm check (one syscall; sync). If data is
    /// ready, do an io_uring Read to fetch and try to complete a frame.
    /// If the Read produces partial bytes (no newline yet), accumulator
    /// retains them and `try_recv` returns `Empty` — a subsequent call
    /// may complete the frame.
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            return decode_frame::<T>(&frame).map_err(|_| TryRecvError::Disconnected);
        }

        let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
            .load(std::sync::atomic::Ordering::SeqCst);
        let read_fd = self.read_fd.as_raw_fd();

        // Non-blocking poll on [data_fd] + [broadcast_fd if initialized].
        // Mirrors typed_channel.rs:431-470 PipeFd typed_try_recv discipline.
        let mut fds = [
            libc::pollfd {
                fd: read_fd,
                events: libc::POLLIN | libc::POLLHUP,
                revents: 0,
            },
            libc::pollfd {
                fd: if broadcast_fd >= 0 { broadcast_fd } else { -1 },
                events: libc::POLLHUP,
                revents: 0,
            },
        ];
        let nfds = if broadcast_fd >= 0 { 2 } else { 1 };

        // SAFETY: fds is a stack-allocated array whose lifetime covers
        // the poll call. libc::poll with timeout=0 returns immediately.
        let n = unsafe { libc::poll(fds.as_mut_ptr(), nfds as libc::nfds_t, 0) };
        if n < 0 {
            return Err(TryRecvError::Disconnected);
        }
        if n == 0 {
            return Err(TryRecvError::Empty);
        }

        // Broadcast wins ties — shutdown overrides any pending Value.
        if nfds == 2 && fds[1].revents != 0 {
            return Err(TryRecvError::Disconnected);
        }
        if fds[0].revents == 0 {
            return Err(TryRecvError::Empty);
        }

        // Data is ready — do ONE io_uring Read. If a complete frame is
        // produced, decode + return Ok(T). If partial bytes only,
        // return Empty (accumulator retains the bytes for next call).
        let mut ring = IoUring::new(2).map_err(|_| TryRecvError::Disconnected)?;
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
                .map_err(|_| TryRecvError::Disconnected)?;
        }
        ring.submit_and_wait(1)
            .map_err(|_| TryRecvError::Disconnected)?;
        let cqe = ring
            .completion()
            .next()
            .ok_or(TryRecvError::Disconnected)?;
        let result = cqe.result();
        if result < 0 {
            return Err(TryRecvError::Disconnected);
        }
        if result == 0 {
            return Err(TryRecvError::Disconnected);
        }
        let bytes_read = result as usize;
        self.accumulator
            .borrow_mut()
            .extend_from_slice(&buf[..bytes_read]);

        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            decode_frame::<T>(&frame).map_err(|_| TryRecvError::Disconnected)
        } else {
            // Partial bytes; no complete frame yet. Caller can retry.
            Err(TryRecvError::Empty)
        }
    }

    /// Count of locally-buffered complete frames in the accumulator.
    ///
    /// APPROXIMATION — the kernel pipe buffer may hold additional bytes
    /// (and additional frames) that aren't visible without consuming
    /// them via `recv` or `try_recv`. Callers needing an exact count
    /// should drain via `try_recv` until `Empty` first; the resulting
    /// `len()` reflects the accumulator only.
    ///
    /// Non-blocking; cascade-irrelevant. Useful for capacity-tracking
    /// callers (e.g., `wat::kernel::HandlePool`) that need a fast
    /// "is anything immediately available?" check.
    pub fn len(&self) -> usize {
        // Count '\n' bytes in the accumulator — each marks the end of
        // a complete frame ready for take_frame to consume.
        self.accumulator.borrow().iter().filter(|&&b| b == b'\n').count()
    }

    /// Signal end-of-stream from this receiver. Consumes self so the
    /// endpoint is gone after close. Other cloned `Receiver` handles
    /// (if any) remain valid. Peer senders see EPIPE on their next
    /// send only after ALL `Receiver` clones close (the pipe's read
    /// reference count hits zero).
    ///
    /// Process-tier close always succeeds — OwnedFd Drop handles
    /// `libc::close(2)`; no fallible operation.
    pub fn close(self) -> Result<(), CloseError> {
        Ok(())
    }
}

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
    /// Panics on `libc::dup` failure (EMFILE/ENFILE; fd table exhausted).
    fn clone(&self) -> Self {
        Self {
            read_fd: self
                .read_fd
                .try_clone()
                .expect("OwnedFd::try_clone (libc::dup) failed — fd table exhausted"),
            accumulator: RefCell::new(Vec::new()),
            _phantom: PhantomData,
        }
    }
}

impl<T: HolonRepresentable> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        Receiver::recv(self)
    }
    fn try_recv(&self) -> Result<T, TryRecvError> {
        Receiver::try_recv(self)
    }
    fn len(&self) -> usize {
        Receiver::len(self)
    }
    fn close(self) -> Result<(), CloseError> {
        Receiver::close(self)
    }
}
```

### 6. Extend `tests/probe_comms_process.rs` with Stone D1 tests

KEEP all 6 existing `probe_slice3c_*` tests unchanged.

ADD 10 new `probe_slice3d1_*` tests:

```rust
// Add to imports at top of file:
use wat::comms::{CommReceiver, CommSender, TryRecvError};

// (Existing imports already include: pair, RecvError, SendError)

// Append AFTER the existing 6 probe_slice3c_* tests:

#[test]
fn probe_slice3d1_try_recv_empty_returns_empty() {
    // Verifies try_recv reports Empty when no data is ready and no
    // shutdown is firing. _tx kept alive so the channel stays
    // connected (Empty, not Disconnected).
    let (_tx, rx) = pair::<String>().expect("pair");
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn probe_slice3d1_try_recv_disconnected_after_sender_drop() {
    // Verifies try_recv reports Disconnected (not Empty) after all
    // senders drop — callers need this distinction to avoid infinite
    // retry loops.
    let (tx, rx) = pair::<String>().expect("pair");
    drop(tx);
    // Give the kernel a moment to propagate the close.
    thread::sleep(Duration::from_millis(20));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn probe_slice3d1_try_recv_succeeds_when_data_ready() {
    // Verifies try_recv returns the value when data is ready.
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("hello".to_string()).expect("send");
    // Give the kernel a moment to flush the write.
    thread::sleep(Duration::from_millis(20));
    let result = rx.try_recv();
    assert_eq!(result, Ok("hello".to_string()));
}

#[test]
fn probe_slice3d1_len_reports_accumulator_frames() {
    // Verifies len() returns the count of complete frames in the
    // accumulator. We don't assert exact intermediate len values
    // (kernel-scheduling dependent) — we verify recv values are
    // correct and len observably tracks consumption.
    let (tx, rx) = pair::<String>().expect("pair");
    assert_eq!(rx.len(), 0, "fresh receiver has empty accumulator");
    tx.send("one".to_string()).expect("send 1");
    tx.send("two".to_string()).expect("send 2");
    // After both recvs, accumulator is drained → len 0.
    assert_eq!(rx.recv().expect("recv 1"), "one");
    assert_eq!(rx.recv().expect("recv 2"), "two");
    assert_eq!(rx.len(), 0, "accumulator empty after both recvs");
}

#[test]
fn probe_slice3d1_sender_close_consumes_endpoint() {
    // Verifies Sender::close consumes self and returns Ok(()).
    let (tx, rx) = pair::<String>().expect("pair");
    let result = tx.close();
    assert_eq!(result, Ok(()));
    drop(rx);
}

#[test]
fn probe_slice3d1_receiver_close_consumes_endpoint() {
    // Verifies Receiver::close consumes self and returns Ok(()).
    let (tx, rx) = pair::<String>().expect("pair");
    let result = rx.close();
    assert_eq!(result, Ok(()));
    drop(tx);
}

#[test]
fn probe_slice3d1_sender_clone_shares_write_end() {
    // Verifies cloned senders both write to the same channel: both
    // values arrive on the receiver. Cloned senders share the kernel
    // pipe via libc::dup.
    let (tx, rx) = pair::<String>().expect("pair");
    let tx2 = tx.clone();
    tx.send("from tx".to_string()).expect("send via tx");
    tx2.send("from tx2".to_string()).expect("send via tx2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    let mut got = [first, second];
    got.sort();
    assert_eq!(got, ["from tx".to_string(), "from tx2".to_string()]);
}

#[test]
fn probe_slice3d1_receiver_clone_competes_for_frames() {
    // Verifies cloned receivers COMPETE for frames — a frame consumed
    // by one clone is gone from the pipe; the other clone sees a
    // DIFFERENT frame (not a duplicate). Cloned receivers share the
    // kernel pipe via libc::dup but each has its own empty accumulator.
    let (tx, rx) = pair::<String>().expect("pair");
    let rx2 = rx.clone();
    tx.send("once".to_string()).expect("send 1");
    tx.send("twice".to_string()).expect("send 2");
    // Both receivers can recv; each gets a DIFFERENT frame.
    let from_rx = rx.recv().expect("recv via rx");
    let from_rx2 = rx2.recv().expect("recv via rx2");
    let mut got = [from_rx, from_rx2];
    got.sort();
    assert_eq!(got, ["once".to_string(), "twice".to_string()]);
}

#[test]
fn probe_slice3d1_comm_sender_trait_dispatch() {
    // Verifies CommSender<T> trait impl works — generic fn over
    // CommSender dispatches correctly to our concrete Sender<T>.
    fn generic_send<S: CommSender<String>>(tx: &S, value: String) -> Result<(), SendError<String>> {
        tx.send(value)
    }
    let (tx, rx) = pair::<String>().expect("pair");
    generic_send(&tx, "via trait".to_string()).expect("send via trait");
    let got = rx.recv().expect("recv");
    assert_eq!(got, "via trait");
}

#[test]
fn probe_slice3d1_comm_receiver_trait_dispatch() {
    // Verifies CommReceiver<T> trait impl works — generic fn over
    // CommReceiver dispatches correctly to our concrete Receiver<T>.
    fn generic_recv<R: CommReceiver<String>>(rx: &R) -> Result<String, RecvError> {
        rx.recv()
    }
    let (tx, rx) = pair::<String>().expect("pair");
    tx.send("via trait".to_string()).expect("send");
    let got = generic_recv(&rx).expect("recv via trait");
    assert_eq!(got, "via trait");
}
```

10 new tests covering: try_recv (3 modes) + len + close (2) + Clone (2) + trait dispatch (2). Total file: 6 (existing) + 10 (new) = 16 tests.

## Verification

```
cargo build --release                                       # MUST be clean
cargo test --release --test probe_comms_process             # 16/16 PASS (6 prior + 10 new)
cargo test --release --test probe_comms_thread              # 10/10 PASS unchanged
cargo test --release --test probe_comms_foundation          # 3/3 PASS unchanged
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts` or any workspace tests.

## Out of scope (STOP triggers)

- **DO NOT implement `Select<'a, T>`** — Stone D2 owns this. If you find yourself drafting Select struct/new/recv/select code, STOP.
- **DO NOT add `ReceiverIndex` or `SelectOutcome` imports** — Stone D2 will add when needed.
- **DO NOT add config tunable** — Stone E
- **DO NOT optimize to persistent IoUring** — Stone E
- **DO NOT modify** Stone A's `take_frame`, Stone B's `wait_for_data_or_cascade` / `PollOutcome`, Stone C's `decode_frame` / `Sender::send` / `Receiver::recv` / `pair<T>()`
- **DO NOT touch the dirty tree** (`src/fork.rs` + `src/spawn_process.rs`)
- **DO NOT touch `src/typed_channel.rs`** (Slice 5 migrates callers later)
- **DO NOT touch** `src/edn_shim.rs` or `src/comms/mod.rs` or `Cargo.toml`
- **DO NOT run** `wat_arc170_program_contracts`
- **DO NOT add HolonRepresentable impls** for any substrate type beyond Stone C's `String`
- **ZERO modifications** outside the 2-file scope (`src/comms/process.rs` adds 5 methods + 2 Clone impls + 2 trait impls; `tests/probe_comms_process.rs` extends with 10 new tests) + SCORE doc

## Pre-emptive ward discipline (lessons from Slices 1+2 + Stones A+B+C)

1. **Module-level + struct doc cascading updates** (Stone B gaze L1 lesson) — replace "NOT Clone / NOT close-able / NOT generic over T" stale claims; reflect actual Stone D1 state. NOTE that Stone D2 is NOT YET shipped — the module doc still names "NO `Select<'a, T>` (Stone D2)" as deferred.
2. **All new `unsafe` blocks carry SAFETY comments** (Stone A round-1 forge lesson) — `try_recv`'s `libc::poll` + io_uring Read submission.
3. **Probe test names** are full imperative sentences (gaze L1 lesson); use `probe_slice3d1_*` prefix.
4. **Sender::send body UNCHANGED** — sonnet must NOT regress Stone C's no-clone-on-error pattern when adding methods around it.
5. **Receiver::recv body UNCHANGED** — Stone B/C's pattern preserved verbatim.
6. **`take_frame` / `wait_for_data_or_cascade` / `decode_frame` UNCHANGED** — sever-clean concern boundaries.
7. **Clone for Receiver gives FRESH empty accumulator** — NOT a clone of the old one. Document explicitly.
8. **`OwnedFd::try_clone` failure → panic via expect** — fd table exhaustion is fail-stop; honest reporting.
9. **`try_recv` on EOF (Read result == 0) returns `Err(TryRecvError::Disconnected)`** — NOT Empty. Callers in retry loops will spin forever otherwise.
10. **Stone D1 = process tier mirror of Slice 2 (sans Select)** — when in doubt about a method's shape, look at `src/comms/thread.rs` for the thread-tier analogue.

## Concrete deliverables list

1. **Edit** `src/comms/process.rs` — module-level doc + Sender/Receiver struct docs updated; new imports added; `Sender::close` + `Clone for Sender<T>` + `CommSender<T>` impl added; `Receiver::try_recv` + `Receiver::len` + `Receiver::close` + `Clone for Receiver<T>` + `CommReceiver<T>` impl added
2. **Edit** `tests/probe_comms_process.rs` — preserve 6 existing `probe_slice3c_*` tests; add 10 new `probe_slice3d1_*` tests; update imports
3. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3D1-MECHANICAL-METHODS.md`

Estimated LOC: ~150-180 LOC added to `src/comms/process.rs`; ~200-240 LOC added to `tests/probe_comms_process.rs`. Total stone delta: ~350-420 LOC.

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + 5-ward pass.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path.
- **Use `git -C`** for any git status / git diff inspections.

## Cross-references

- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — Stone A foundation
- BRIEF-214-SLICE-3B-CASCADE-AWARE-MULTI-ARM.md — Stone B cascade
- BRIEF-214-SLICE-3C-HOLON-REPRESENTABLE.md — Stone C generic-T
- WARD-PASS-3A through 3C — prior round-trips
- `src/comms/thread.rs` — Slice 2 thread tier MIRROR REFERENCE
- `src/comms/mod.rs:65-105` — Slice 1 `CommSender<T>` + `CommReceiver<T>` traits
- `src/typed_channel.rs:407-470` — existing PipeFd `typed_try_recv` pattern (READ-ONLY reference)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol
- `feedback_iterative_complexity` — Stone D split from one stone into D1 + D2 per the four-questions test

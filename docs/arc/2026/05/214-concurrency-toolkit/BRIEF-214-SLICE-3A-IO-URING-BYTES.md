# Arc 214 Slice 3 — Stone A — io_uring bytes proof of life

## Mission

Stone A is the FIRST of five stepping stones in Slice 3 (process tier). It proves **one** thing: the io-uring crate works in our substrate for byte-oriented pipe I/O. Nothing more.

This stone deliberately defers:
- Cascade-aware multi-arm (Stone B)
- Generic `T: HolonRepresentable` serialization (Stone C)
- `try_recv` + `Select` + Clone + close + len + trait impls (Stone D)
- Persistent IoUring + config tunable (Stone E)

Per the kernel-impeccability protocol: isolating io_uring API correctness in its own stone means a Stone A defect can be ONLY an io_uring API misuse — no cascade-or-serialization ambiguity. Subsequent stones extend this proven foundation; each carries its own gate.

## Stepping stone roadmap (Slice 3; informational — do not implement beyond Stone A)

- **Stone A (this work):** io_uring bytes-only `pair()` + `Sender::send(&[u8])` + `Receiver::recv() -> Result<Vec<u8>, RecvError>` with newline framing. Per-call IoUring (long-lived ring is Stone E). NO cascade. NO generics. NO trait impls.
- **Stone B:** Add cascade-aware multi-arm POLL_ADD on `[data_fd, broadcast_fd]`; broadcast wins ties; bootstrap fallback.
- **Stone C:** Make Sender/Receiver generic over `T: HolonRepresentable`; HolonAST ↔ EDN bytes via wat-edn; `impl HolonRepresentable for String`.
- **Stone D:** `try_recv` + `Select<'a, T>` + Clone + close + len + CommSender/CommReceiver trait impls.
- **Stone E:** Persistent IoUring per Receiver (via interior mutability) + `:wat::config::set-process-tier-uring-depth!` (default 512, [1, 4096], power of 2).

## Substrate context (substrate-truth verified pre-spawn)

- **`io-uring` crate NOT yet in deps.** Stone A adds it. Use version `0.7`; default features fine (no SQPOLL, no kernel-version gating needed for our usage).
- **`libc::pipe(2)`** is the canonical anonymous-pipe primitive; returns `[read_fd, write_fd]`; both are blocking-mode by default (acceptable for io_uring since we drive readiness via SQE).
- **`std::os::fd::OwnedFd`** owns a file descriptor + closes on Drop. Use `unsafe { OwnedFd::from_raw_fd(fd) }` after `libc::pipe`. NEVER `forget()` an OwnedFd (leaks the fd).
- **Pattern reference (READ-ONLY):** `src/typed_channel.rs:324-388` is the existing `libc::poll`-based PipeFd recv. Stone B (NOT Stone A) replaces this with io_uring. Stone A does NOT touch typed_channel.rs.
- **Slice 1 error types available at `crate::comms::*`:** Stone A uses `SendError<Vec<u8>>` and `RecvError`. No new error types in Stone A.

## Concrete deliverables

### 1. Add `io-uring` dep to `Cargo.toml`

Locate the existing `[dependencies]` section (line ~48). Add `io-uring` adjacent to `crossbeam-channel` and `libc` (alphabetical-ish; group with low-level concurrency primitives):

```toml
crossbeam-channel = "0.5"
io-uring = "0.7"
libc = "0.2"
```

(Single-line addition. No feature flags. No optional. No build-script gating.)

### 2. Create `src/comms/process.rs`

Full skeleton (~150 LOC). Sonnet's job: type the code below + write 6 probe tests. Judgment calls minimized.

```rust
//! # Process tier — cross-process comms via io_uring + anonymous pipes
//!
//! Layer 0a tier implementation per arc 214's `DESIGN.md`. Builds on the
//! Slice 1 traits (`crate::comms::{SendError, RecvError}`) using
//! `libc::pipe` for the transport and `io_uring` for the wake mechanism.
//!
//! ## Stone A scope (this commit)
//!
//! Bytes-only proof of life. `Sender::send(&[u8])` writes
//! newline-framed bytes; `Receiver::recv() -> Result<Vec<u8>, RecvError>`
//! reads one newline-framed frame via io_uring. NO cascade-aware
//! multi-arm (Stone B); NO generic `T: HolonRepresentable` (Stone C);
//! NO try_recv / Select / Clone / close / len / trait impls (Stone D);
//! NO persistent ring / config tunable (Stone E).
//!
//! ## Framing (Stone A)
//!
//! Each `send` appends `'\n'` to its payload and writes the framed bytes
//! to the pipe atomically (writes ≤ PIPE_BUF = 4096 are atomic per
//! POSIX). The receiver reads bytes into an internal accumulator and
//! splits on `'\n'`; any tail bytes after the first newline are kept
//! for the next `recv` call.
//!
//! Payload bytes MUST NOT contain `'\n'` in Stone A (caller-enforced;
//! Stone C migrates to length-prefixed EDN bytes which removes this
//! constraint). Stone A test payloads are ASCII strings.
//!
//! ## Cascade contract (NOT WIRED IN STONE A)
//!
//! Stone B wires `SHUTDOWN_BROADCAST_READ_FD` as a second POLL_ADD arm
//! so that substrate shutdown wakes blocked recvs. Stone A's `recv`
//! WILL hang if the substrate shuts down before a frame arrives — this
//! is acceptable for Stone A because callers don't yet use this tier
//! in production paths.
//!
//! ## Audience
//!
//! Substrate-internal Rust code (Stone D's `Select`, Stone E's tunable,
//! Slice 4's kernel dispatcher). User code does NOT touch this tier.

use std::cell::RefCell;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use io_uring::{opcode, types, IoUring};

use crate::comms::{RecvError, SendError};

// ─── Sender ──────────────────────────────────────────────────────────────────

/// Stone-A process-tier send endpoint. Owns the pipe's write-end fd.
/// Writes newline-framed bytes synchronously via `libc::write`.
///
/// Stone A: NOT Clone (Stone D adds Clone). NOT close-able (Stone D adds
/// `close(self)`). Drop closes the fd automatically (OwnedFd Drop impl).
#[derive(Debug)]
pub struct Sender {
    write_fd: OwnedFd,
}

impl Sender {
    /// Send `bytes` to the channel as a newline-framed frame. Writes
    /// `bytes + '\n'` to the pipe via a `libc::write` retry loop.
    ///
    /// Returns `Err(SendError(bytes.to_vec()))` when the peer's read-end
    /// is closed (EPIPE) or when the write fails for any other reason
    /// (rare; non-EINTR I/O error). The error carries the bytes so the
    /// caller can recover or re-send.
    ///
    /// Bytes MUST NOT contain `'\n'` in Stone A (caller-enforced framing
    /// constraint; Stone C removes this when EDN serialization replaces
    /// newline framing).
    pub fn send(&self, bytes: &[u8]) -> Result<(), SendError<Vec<u8>>> {
        // Frame: payload + '\n'. Single allocation; single contiguous write.
        let mut framed: Vec<u8> = Vec::with_capacity(bytes.len() + 1);
        framed.extend_from_slice(bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        let mut written = 0usize;
        while written < framed.len() {
            let n = unsafe {
                libc::write(
                    fd,
                    framed[written..].as_ptr() as *const _,
                    framed.len() - written,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // EPIPE (peer closed) or other write failure — caller
                // gets the bytes back.
                return Err(SendError(bytes.to_vec()));
            }
            written += n as usize;
        }
        Ok(())
    }
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// Stone-A process-tier receive endpoint. Owns the pipe's read-end fd
/// and a small internal byte accumulator for cross-call frame splitting.
///
/// Stone A: NOT Clone (Stone D). NOT cascade-aware (Stone B). NOT generic
/// over `T` (Stone C). Per-call `IoUring` instance (Stone E persistifies).
#[derive(Debug)]
pub struct Receiver {
    read_fd: OwnedFd,
    /// Bytes read from the pipe but not yet returned to a caller.
    /// `RefCell` provides interior mutability so `recv(&self)` can
    /// update the accumulator without `&mut self`. `Receiver` is `!Sync`
    /// by construction (RefCell is !Sync); the substrate's threading
    /// model never shares a single Receiver across threads — clones
    /// (Stone D) create independent endpoints.
    accumulator: RefCell<Vec<u8>>,
}

impl Receiver {
    /// Blocking recv. Returns the next complete newline-framed frame
    /// from the pipe (without the trailing `'\n'`). Reads from the
    /// internal accumulator first; if no complete frame is buffered,
    /// drives io_uring single-arm Read until a `'\n'` is observed.
    ///
    /// Returns `Err(RecvError)` on peer-close (EOF; read returns 0)
    /// or on io_uring submission/completion failure.
    ///
    /// Stone A: NOT cascade-aware. If `SHUTDOWN_BROADCAST_READ_FD`
    /// fires while a recv is blocked, this call WILL HANG until the
    /// pipe also produces a frame or closes. Stone B wires the
    /// broadcast arm.
    pub fn recv(&self) -> Result<Vec<u8>, RecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            return Ok(frame);
        }

        // Slow path — io_uring loop: read more bytes; check accumulator;
        // repeat until a complete frame is available OR EOF is reached.
        loop {
            // Per-call IoUring (Stone A simplification; Stone E persistifies).
            // Ring size = 2 entries — we only ever have one Read in flight
            // per loop iteration; 2 gives slack for the kernel's bookkeeping.
            let mut ring = IoUring::new(2).map_err(|_| RecvError)?;
            let mut buf = [0u8; 4096];
            let read_e = opcode::Read::new(
                types::Fd(self.read_fd.as_raw_fd()),
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
                // I/O error.
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

            // Check whether we now have a complete frame.
            if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
                return Ok(frame);
            }
            // No complete frame yet; loop and read more bytes.
        }
    }
}

/// Pull the first newline-terminated frame out of `acc` (consuming the
/// frame bytes + the trailing `'\n'`). Returns `None` when no `'\n'`
/// is present (caller should read more bytes).
fn take_frame(acc: &mut Vec<u8>) -> Option<Vec<u8>> {
    let pos = acc.iter().position(|&b| b == b'\n')?;
    // Split acc: [0..=pos] becomes the frame (with trailing \n);
    // [pos+1..] becomes the new accumulator content.
    let suffix = acc.split_off(pos + 1);
    let mut frame = std::mem::replace(acc, suffix);
    frame.pop(); // remove trailing '\n'
    Some(frame)
}

// ─── Factory ─────────────────────────────────────────────────────────────────

/// Create a new process-tier channel pair (Stone A — bytes only).
///
/// Allocates an anonymous pipe via `libc::pipe(2)` and wraps the two
/// file descriptors as `Sender` / `Receiver`. Returns the OS-level
/// `io::Error` on `pipe(2)` failure (rare; out of fds or kernel OOM).
pub fn pair() -> std::io::Result<(Sender, Receiver)> {
    let mut fds = [0i32; 2];
    let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: pipe(2) returned two valid, owned fds. Wrap each as OwnedFd
    // so Drop closes them; never call OwnedFd::from_raw_fd on the same
    // fd twice (would double-close).
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((
        Sender { write_fd },
        Receiver {
            read_fd,
            accumulator: RefCell::new(Vec::new()),
        },
    ))
}
```

### 3. Wire up `pub mod process;` in `src/comms/mod.rs`

Edit `src/comms/mod.rs`: add `pub mod process;` immediately after the existing `pub mod thread;` declaration (line ~177; the tier-module section). Match the surrounding doc-comment style:

```rust
/// Thread tier: in-process comms via crossbeam_channel. Cascade-aware.
/// Substrate-internal; user code uses `:wat::kernel::*` verbs (Slice 4).
pub mod thread;

/// Process tier: cross-process comms via io_uring + anonymous pipes.
/// Cascade-aware (Stone B). Substrate-internal; user code uses
/// `:wat::kernel::*` verbs (Slice 4).
pub mod process;
```

ONE-LINE addition + 3-line doc comment. No other edits to mod.rs.

### 4. Create `tests/probe_comms_process.rs`

Six smoke tests with REAL assertions (no `_`-bindings without follow-up checks — per Slice 1 gaze L1 + Slice 2 gaze L2 lessons). Each test name is a full imperative statement of what it proves.

```rust
//! Arc 214 Slice 3 Stone A smoke probe — verify io_uring bytes round-trip.
//!
//! Six tests:
//!   1. pair() constructs successfully
//!   2. single-frame round-trip preserves bytes
//!   3. FIFO ordering across multiple sends
//!   4. sender drop wakes recv with Err(RecvError)
//!   5. accumulator correctly splits two frames received in one read
//!   6. large frame spans multiple io_uring reads
//!
//! Stone A is NOT cascade-aware (Stone B). Tests do NOT exercise
//! substrate shutdown — they exercise io_uring + framing only.

use std::thread;
use std::time::Duration;

use wat::comms::{RecvError, SendError};
use wat::comms::process::{pair, Sender, Receiver};

#[test]
fn probe_slice3a_pair_constructs_successfully() {
    // Verifies the libc::pipe → OwnedFd wrapping path works at all.
    let result = pair();
    assert!(result.is_ok(), "pair() must return Ok; got {:?}", result.err());
    let (_tx, _rx) = result.expect("pair");
    // Drop closes both fds; no fd leak. (Verified statically by OwnedFd
    // Drop impl; not asserted at runtime.)
}

#[test]
fn probe_slice3a_single_frame_round_trip() {
    // Verifies the core contract: bytes sent are bytes received.
    let (tx, rx) = pair().expect("pair");
    tx.send(b"hello").expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent frame");
    assert_eq!(got, b"hello", "received bytes must equal sent bytes");
}

#[test]
fn probe_slice3a_fifo_ordering_preserved_across_sends() {
    // Verifies that N sends followed by N recvs preserve order.
    let (tx, rx) = pair().expect("pair");
    tx.send(b"first").expect("send 1");
    tx.send(b"second").expect("send 2");
    tx.send(b"third").expect("send 3");
    assert_eq!(rx.recv().expect("recv 1"), b"first");
    assert_eq!(rx.recv().expect("recv 2"), b"second");
    assert_eq!(rx.recv().expect("recv 3"), b"third");
}

#[test]
fn probe_slice3a_sender_drop_wakes_recv_with_err() {
    // Verifies that dropping the Sender causes recv to return Err(RecvError)
    // (EOF on the pipe; io_uring Read returns 0). The Sender is dropped
    // on a separate thread to ensure recv() is genuinely blocked when the
    // drop happens.
    let (tx, rx) = pair().expect("pair");
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        drop(tx);
    });
    let result = rx.recv();
    assert_eq!(
        result,
        Err(RecvError),
        "recv must return Err(RecvError) after sender drop (EOF)"
    );
    handle.join().expect("sender-drop thread");
}

#[test]
fn probe_slice3a_accumulator_splits_two_frames_from_one_read() {
    // Verifies the accumulator correctness: when the sender writes two
    // newline-framed payloads in one libc::write call (kernel delivers
    // both atomically), the first recv() returns frame 1 and the second
    // recv() returns frame 2 WITHOUT another io_uring read.
    //
    // This exercises the take_frame fast path on the second call.
    let (tx, rx) = pair().expect("pair");
    // Two sends in quick succession; the kernel typically merges them
    // into one available chunk on the read side (PIPE_BUF atomicity).
    tx.send(b"alpha").expect("send 1");
    tx.send(b"beta").expect("send 2");
    let first = rx.recv().expect("recv 1");
    let second = rx.recv().expect("recv 2");
    assert_eq!(first, b"alpha", "first recv must return first frame");
    assert_eq!(second, b"beta", "second recv must return second frame");
}

#[test]
fn probe_slice3a_large_frame_spans_multiple_io_uring_reads() {
    // Verifies that a frame larger than the io_uring read buffer (4096)
    // is correctly assembled across multiple loop iterations of recv().
    // Build a 10_000-byte payload (no newlines per Stone A framing
    // constraint).
    let (tx, rx) = pair().expect("pair");
    let payload: Vec<u8> = (0..10_000u32).map(|i| (i % 26) as u8 + b'a').collect();
    // Sender on a separate thread because a 10001-byte write may block
    // if the pipe buffer is full (typical pipe buffer = 64KB so this is
    // fine; the thread split also exercises the recv-blocks-then-wakes
    // path).
    let payload_clone = payload.clone();
    let send_handle = thread::spawn(move || {
        tx.send(&payload_clone).expect("send large");
    });
    let got = rx.recv().expect("recv large");
    assert_eq!(got.len(), payload.len(), "received length must match sent");
    assert_eq!(got, payload, "received bytes must equal sent bytes");
    send_handle.join().expect("sender thread");
}
```

## Verification

```
cargo build --release                                       # MUST be clean (no new warnings)
cargo test --release --test probe_comms_process             # 6/6 PASS
cargo test --release --test probe_comms_thread              # 10/10 PASS unchanged
cargo test --release --test probe_comms_foundation          # 3/3 PASS unchanged (Slice 1)
cargo test --release --test probe_channel_primitive         # 3/3 PASS unchanged (χ-1)
cargo test --release --test probe_pidfd_primitive           # 2/2 PASS unchanged (α)
```

Per `feedback_no_hang_vector_in_additive_scorecard`: **DO NOT** run `wat_arc170_program_contracts` or any workspace tests. Stone A is additive; cargo-build-clean + the new probe + unchanged-prior-probes is the verification surface.

## Out of scope (STOP triggers)

- **DO NOT add cascade-aware multi-arm** — Stone B owns the `SHUTDOWN_BROADCAST_READ_FD` integration
- **DO NOT make Sender/Receiver generic over T** — Stone C owns the HolonRepresentable serialization layer
- **DO NOT implement try_recv** — Stone D owns the non-blocking API
- **DO NOT implement Select** — Stone D owns the fan-in API
- **DO NOT implement Clone** — Stone D owns clone semantics (needs OwnedFd dup discipline)
- **DO NOT implement close(self)** — Stone D owns close semantics
- **DO NOT implement len()** — Stone D owns this (possibly via FIONREAD ioctl)
- **DO NOT implement CommSender / CommReceiver trait impls** — Stone D owns these (they require the full surface)
- **DO NOT add config tunable** — Stone E owns the `:wat::config::set-process-tier-uring-depth!` setter
- **DO NOT optimize to persistent IoUring** — Stone E owns this (requires interior mutability decision)
- **DO NOT add manual HolonRepresentable impls** for substrate types — Slice 4/5 territory
- **DO NOT touch the dirty tree** (`src/fork.rs` + `src/spawn_process.rs` — arc 213 δ-1)
- **DO NOT touch `src/typed_channel.rs`** (existing PipeFd; Slice 5 migrates callers later)
- **DO NOT run `wat_arc170_program_contracts` or workspace tests** (per additive-scorecard discipline)
- **ZERO modifications** outside the 4-file scope listed below

## Pre-emptive ward discipline (lessons from Slices 1 + 2)

1. **All public items get doc comments** (gaze L2 lesson). Every `pub fn` / `pub struct` / `pub mod process;` doc-comment.
2. **Tests have REAL assertions** (gaze L1 lesson). No bare `_`-bindings without follow-up `assert_eq!` / `assert!(matches!)`. Test names are full imperative statements.
3. **Newtype + accessor pattern** for error-message-style types (forge L1 lesson). Stone A uses Slice 1's existing `SendError<T>` / `RecvError` as-is; no re-minted error wrappers.
4. **Comments explain WHY not WHAT** (gaze L2 lesson). The doc comments on `Sender::send`, `Receiver::recv`, `take_frame`, and `pair` all explain WHY (the framing constraint, the cascade-NOT-wired note, the OwnedFd ownership rule, the per-call IoUring rationale).
5. **`rune:forge(escape)` annotations** at any algebraic-escape sites (forge L1 lesson from Slice 2). Stone A's `unsafe` blocks (`libc::pipe`, `libc::write`, `OwnedFd::from_raw_fd`, io_uring submission) are NOT algebraic escapes (they're FFI boundaries to libc/io_uring, not substrate-state reach-outs). The SAFETY comment on each `unsafe` block is the honest rune for FFI escapes.
6. **Honest scope** (Slice 2 reap lesson). Sonnet's honest-delta declaration if anything beyond BRIEF scope is added. Do NOT add `is_empty`-style convenience methods beyond what BRIEF lists.

## Concrete deliverables list

1. **Edit** `Cargo.toml` — add `io-uring = "0.7"` line in `[dependencies]` (~line 53-55 region)
2. **New file** `src/comms/process.rs` (~150-180 LOC: Sender + Receiver + take_frame + pair + module doc per skeleton above)
3. **Edit** `src/comms/mod.rs` — add `pub mod process;` block at end of file mirroring the existing `pub mod thread;` declaration
4. **New file** `tests/probe_comms_process.rs` (~120-150 LOC: 6 smoke tests per skeleton above)
5. **New file** SCORE doc: `docs/arc/2026/05/214-concurrency-toolkit/SCORE-214-SLICE-3A-IO-URING-BYTES.md` (sonnet writes after work)

## Critical constraints

- **DO NOT commit.** Orchestrator commits after SCORE verification + 5-ward pass per kernel-impeccability protocol.
- **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` — `pwd` as first action; reject any `.claude/worktrees/` path.
- **Use `git -C`** for any git status / git diff inspections.

## Cross-references

- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` — full arc 214 design; Slice 3 description
- `docs/arc/2026/05/214-concurrency-toolkit/WARD-PASS-1-FOUNDATION-PRIMITIVES.md` — Slice 1 ward round-trip; lessons pre-empted
- `docs/arc/2026/05/214-concurrency-toolkit/WARD-PASS-2-THREAD-TIER.md` — Slice 2 ward round-trip; 5-ward protocol established
- `src/typed_channel.rs:324-388` — existing PipeFd recv pattern (READ-ONLY reference; Stone B will replace this in spirit)
- `src/runtime.rs:201` — `SHUTDOWN_BROADCAST_READ_FD` (NOT used in Stone A; Stone B integrates)
- `src/comms/mod.rs` — Slice 1 traits + error types
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `feedback_no_hang_vector_in_additive_scorecard` — why no wat_arc170 in verification
- `feedback_defect_fix_or_panic_never_revert` — dirty tree preservation
- `feedback_iterative_complexity` — stepping stone discipline; why 5 stones not 3
- `feedback_simple_is_uniform_composition` — Stone A is one concern (io_uring API correctness); subsequent stones extend

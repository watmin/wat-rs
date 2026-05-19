//! # Process tier — cross-process comms via io_uring + anonymous pipes
//!
//! Layer 0a tier implementation per arc 214's `DESIGN.md`. Builds on the
//! Slice 1 traits (`crate::comms::{SendError, RecvError}`) using
//! `libc::pipe` for the transport and `io_uring` for the wake mechanism.
//!
//! ## Current scope (through Stone C)
//!
//! Generic `Sender<T: HolonRepresentable>` / `Receiver<T: HolonRepresentable>`
//! with HolonAST ↔ EDN bytes via wat-edn (Stone C). Cascade-aware
//! multi-arm POLL_ADD (Stone B). io_uring bytes foundation with
//! newline framing (Stone A). Still NO try_recv / Select / Clone /
//! close / len / trait impls (Stone D); NO persistent ring / config
//! tunable (Stone E).
//!
//! ## Framing
//!
//! Each `send` encodes `T` as a tagged-EDN single-line string via
//! `write_holon_ast_tagged`, appends `'\n'`, and writes atomically
//! (writes ≤ PIPE_BUF = 4096 are atomic per POSIX). The receiver
//! reads bytes into an internal accumulator and splits on `'\n'`;
//! the trailing newline does not appear in EDN output because
//! wat-edn produces single-line text (embedded newlines escape as
//! `\n` literal). Frames are decoded back via
//! `read_holon_ast_tagged` + `T::from_holon_ast`.
//!
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
//!
//! ## Audience
//!
//! Substrate-internal Rust code (Stone D's `Select`, Stone E's tunable,
//! Slice 4's kernel dispatcher). User code does NOT touch this tier.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

use io_uring::{opcode, types, IoUring};

use crate::comms::{HolonRepresentable, RecvError, SendError};

// ─── Sender ──────────────────────────────────────────────────────────────────

/// Process-tier send endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's write-end fd. Encodes `T` via
/// `HolonRepresentable::to_holon_ast` → `write_holon_ast_tagged` →
/// newline-framed bytes.
///
/// NOT Clone (Stone D adds). NOT close-able (Stone D adds `close(self)`).
/// Drop closes the fd automatically (OwnedFd Drop impl).
#[derive(Debug)]
pub struct Sender<T: HolonRepresentable> {
    write_fd: OwnedFd,
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `send` accepts. `PhantomData<T>` makes `Sender<T>` invariant
    /// in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: HolonRepresentable> Sender<T> {
    /// Send `value` to the channel. Encodes via
    /// `T::to_holon_ast` → `edn_shim::write_holon_ast_tagged` →
    /// newline-framed bytes → `libc::write` retry loop.
    ///
    /// Returns `Err(SendError(value))` when the peer's read-end is
    /// closed (EPIPE) or when the write fails for any other reason.
    /// The error carries the original `T` so the caller can recover
    /// or re-send.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Encode T → HolonAST → tagged EDN string (single-line).
        let ast = value.to_holon_ast();
        let edn_str = crate::edn_shim::write_holon_ast_tagged(&ast);

        // Frame: EDN bytes + '\n'. Single allocation; single contiguous write.
        let edn_bytes = edn_str.as_bytes();
        let mut framed: Vec<u8> = Vec::with_capacity(edn_bytes.len() + 1);
        framed.extend_from_slice(edn_bytes);
        framed.push(b'\n');

        let fd = self.write_fd.as_raw_fd();
        let mut written = 0usize;
        while written < framed.len() {
            // SAFETY: `fd` is valid for the lifetime of `self.write_fd`
            // (OwnedFd-managed; not closed until Drop). The pointer
            // derived from `framed[written..]` is valid for
            // `framed.len() - written` bytes — `framed` is a live Vec
            // on this function's stack and is not freed until after
            // this loop completes.
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
                // gets the original value back.
                return Err(SendError(value));
            }
            written += n as usize;
        }
        Ok(())
    }
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// Process-tier receive endpoint. Generic over the payload type T (Stone C).
/// Owns the pipe's read-end fd and a small internal byte accumulator
/// for cross-call frame splitting.
///
/// Cascade-aware (Stone B): `recv` wakes on substrate shutdown via
/// io_uring multi-arm POLL_ADD on `SHUTDOWN_BROADCAST_READ_FD`.
/// NOT Clone (Stone D adds). Per-call `IoUring` instance (Stone E
/// persistifies).
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
    /// Type marker — `T` doesn't appear in any field but constrains
    /// what `recv` produces. `PhantomData<T>` makes `Receiver<T>`
    /// invariant in T which is correct for this use case.
    _phantom: PhantomData<T>,
}

impl<T: HolonRepresentable> Receiver<T> {
    /// Blocking recv. Returns the next complete `T` decoded from the
    /// pipe (newline-framed; EDN-encoded). Reads from the internal
    /// accumulator first; if no complete frame is buffered, drives
    /// the cascade-aware io_uring multi-arm POLL_ADD + Read loop
    /// until a `'\n'` is observed; then decodes the frame via
    /// `read_holon_ast_tagged` + `T::from_holon_ast`.
    ///
    /// Returns `Err(RecvError)` on peer-close (EOF; read returns 0),
    /// on io_uring submission/completion failure, on substrate
    /// shutdown (cascade-arm fires; Stone B), on UTF-8 decode failure,
    /// on EDN parse failure, or on `T::from_holon_ast` failure.
    pub fn recv(&self) -> Result<T, RecvError> {
        // Fast path — accumulator already has a complete frame.
        if let Some(frame) = take_frame(&mut self.accumulator.borrow_mut()) {
            return decode_frame::<T>(&frame);
        }

        let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD
            .load(std::sync::atomic::Ordering::SeqCst);
        let read_fd = self.read_fd.as_raw_fd();

        loop {
            // Cascade-aware step — poll both arms (data + broadcast).
            // Bootstrap fallback: when broadcast_fd is -1 (pre-init or
            // test bypass), skip the poll and fall through to bare Read
            // (Stone A behavior; no cascade available).
            if broadcast_fd >= 0 {
                match wait_for_data_or_cascade(read_fd, broadcast_fd)? {
                    PollOutcome::Shutdown => return Err(RecvError),
                    PollOutcome::DataReady => {
                        // Data is ready; fall through to Read step.
                    }
                }
            }

            // Read step — same as Stones A+B. Per-call IoUring; ring size 2.
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
                return decode_frame::<T>(&frame);
            }
            // No complete frame yet; loop and poll/read more bytes.
        }
    }
}

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
    const BROADCAST_TOKEN: u64 = 2;

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
    .user_data(BROADCAST_TOKEN);

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

/// Decode a newline-framed payload to `T` via the Stone C wire chain:
/// UTF-8 bytes → tagged-EDN string → HolonAST → T.
///
/// Returns `Err(RecvError)` on any layer's failure (utf8, EDN parse,
/// or `T::from_holon_ast`). The error type collapses all three causes
/// because the caller cannot meaningfully distinguish them — wire
/// failures all mean "the frame did not roundtrip cleanly; the channel
/// is in an honest but unrecoverable state per this call".
fn decode_frame<T: HolonRepresentable>(bytes: &[u8]) -> Result<T, RecvError> {
    let s = std::str::from_utf8(bytes).map_err(|_| RecvError)?;
    let ast_arc = crate::edn_shim::read_holon_ast_tagged(s).map_err(|_| RecvError)?;
    T::from_holon_ast(&ast_arc).map_err(|_| RecvError)
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

/// Create a new process-tier channel pair (Stone C — generic over T).
///
/// Allocates an anonymous pipe via `libc::pipe(2)` and wraps the two
/// file descriptors as `Sender<T>` / `Receiver<T>`. The type parameter
/// `T` constrains what values flow through the channel; both endpoints
/// must agree on `T` (typically inferred at call site).
///
/// Returns the OS-level `io::Error` on `pipe(2)` failure (rare; out
/// of fds or kernel OOM).
pub fn pair<T: HolonRepresentable>() -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let mut fds = [0i32; 2];
    // SAFETY: `fds` is a valid `[i32; 2]` stack allocation whose
    // lifetime covers this call; `libc::pipe` writes two file
    // descriptors into it.
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
        Sender {
            write_fd,
            _phantom: PhantomData,
        },
        Receiver {
            read_fd,
            accumulator: RefCell::new(Vec::new()),
            _phantom: PhantomData,
        },
    ))
}

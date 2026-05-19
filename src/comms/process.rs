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
        Sender { write_fd },
        Receiver {
            read_fd,
            accumulator: RefCell::new(Vec::new()),
        },
    ))
}

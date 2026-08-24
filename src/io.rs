//! wat IO substrate — `:wat::io::IOReader` + `:wat::io::IOWriter` abstractions.
//!
//! Wat needs substitutable stdio: in production, a wat program receives
//! real OS Stdin/Stdout/Stderr; in tests, the same program receives
//! string-buffer stand-ins. Both must fit a single wat-level type so the
//! source is identical. Ruby's StringIO model.
//!
//! Rust's `Read` / `Write` are separate traits — different
//! responsibilities. Wat mirrors that split: `IOReader` is what stdin
//! fits; `IOWriter` is what stdout / stderr fit. A wat program that
//! tries to write to stdin fails at check time, not runtime.
//!
//! Concrete impls (in this module):
//!
//! - [`RealStdin`] / [`RealStdout`] / [`RealStderr`] — wrap Rust's
//!   stdlib `std::io::Stdin` / `Stdout` / `Stderr` via `Arc`. Rust's
//!   stdlib handles its own internal locking; wat-rs introduces no
//!   new Mutex.
//! - [`StringIoReader`] / [`StringIoWriter`] — `ThreadOwnedCell`-backed
//!   in-memory stand-ins. Single-thread-owned; cross-thread use panics
//!   with the owner-check error (matches the tier-2 `LocalCache`
//!   pattern). Zero Mutex. All IO calls synchronous on caller's thread
//!   — no channel round-trip, no driver spawn.
//!
//! This substrate is arc 008; arc 007's `run-sandboxed` and
//! `:wat::test::*` sit on top.

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::TrackedValue;
use crate::rust_deps::ThreadOwnedCell;
use crate::span::Span;
use std::sync::Arc;
use wat_macros::restricted_to;

// ─── Traits ──────────────────────────────────────────────────────────────

/// A source of bytes. Wat-level type `:wat::io::IOReader`.
pub trait WatReader: Send + Sync + std::fmt::Debug {
    /// Read up to `n` bytes. Returns `Ok(None)` on EOF, `Ok(Some(bytes))`
    /// with the actual bytes (may be fewer than `n`). I/O errors and
    /// owner-check failures surface as `RuntimeError`.
    fn read(&self, n: usize, span: Span) -> Result<Option<Vec<u8>>, RuntimeError>;

    /// Read until EOF. Returns every byte in order.
    fn read_all(&self, span: Span) -> Result<Vec<u8>, RuntimeError>;

    /// Read one line (up to and including `\n`, which is consumed but
    /// not returned). Returns `Ok(None)` on EOF. The string is
    /// UTF-8-decoded; invalid bytes surface as a `MalformedForm` error.
    fn read_line(&self, span: Span) -> Result<Option<String>, RuntimeError>;

    /// Reset the read cursor to the start of the backing source. No-op
    /// for real stdin (real fds aren't rewindable); meaningful for
    /// `StringIoReader`.
    fn rewind(&self, span: Span) -> Result<(), RuntimeError>;

    /// Arc 170 Phase 2 — return the raw FD this reader is backed by,
    /// for OS-level multiplex via poll(2). Default `None` for
    /// non-FD-backed readers (StringIoReader). Override in PipeReader
    /// to return `Some(self.fd.as_raw_fd())`, and in RealStdin to
    /// return `Some(0)` (it wraps `std::io::Stdin`, which is fd 0 —
    /// see the arc 170 stdin-joins-the-lockstep fix).
    fn as_raw_fd_for_poll(&self) -> Option<i32> {
        None
    }
}

/// A sink for bytes. Wat-level type `:wat::io::IOWriter`.
pub trait WatWriter: Send + Sync + std::fmt::Debug {
    /// Write up to `bytes.len()` bytes. Returns the count actually
    /// written. Matches Rust `Write::write` semantics (fd-honest
    /// partial writes).
    fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError>;

    /// Write all `bytes`. Loops internally if a single write is
    /// partial. Matches Rust `Write::write_all`.
    fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError>;

    /// Flush any buffered output.
    fn flush(&self, span: Span) -> Result<(), RuntimeError>;

    /// Clone the writer's accumulated bytes, if the impl backs to an
    /// in-memory buffer. `None` for real stdio (the OS pipe's past is
    /// not inspectable). `Some(bytes)` for `StringIoWriter`.
    /// Used by `:wat::io::IOWriter/to-bytes` and
    /// `/to-string` — callers that need to capture what the sandboxed
    /// program wrote.
    fn snapshot(&self) -> Option<Vec<u8>> {
        None
    }

    /// Idempotent close. Default no-op for backings without an
    /// explicit-close concept (StringIoWriter, RealStdout, RealStderr).
    /// Pipe-backed writers override to release the fd early so the
    /// peer reader sees EOF without waiting for the Arc count to
    /// reach zero — needed when the writer Arc is held by a struct
    /// (e.g., `:wat::kernel::Process.stdin`) that outlives the write
    /// phase. Subsequent writes to a closed writer return an error.
    fn close(&self, _span: Span) -> Result<(), RuntimeError> {
        Ok(())
    }

    /// The raw fd this writer backs, if any — for the arc 170 stdio-as-defservice bootstrap, which
    /// seeds the primed services with fd NUMBERS. `None` for non-FD-backed writers (RealStdout,
    /// StringIoWriter); overridden in PipeWriter to return `Some(self.fd)`. Mirrors the WatReader
    /// method of the same name (io.rs:62) — symmetry the writer trait was missing.
    fn as_raw_fd_for_poll(&self) -> Option<i32> {
        None
    }
}

// ─── Real stdio wrappers ─────────────────────────────────────────────────

/// Wraps Rust's `std::io::Stdin`. Thread-safe via Rust stdlib's internal
/// locking; wat-rs introduces no Mutex.
#[derive(Debug)]
pub struct RealStdin {
    pub(crate) inner: Arc<std::io::Stdin>,
}

impl RealStdin {
    pub fn new(inner: Arc<std::io::Stdin>) -> Self {
        Self { inner }
    }
}

impl WatReader for RealStdin {
    fn read(&self, n: usize, span: Span) -> Result<Option<Vec<u8>>, RuntimeError> {
        use std::io::Read;
        let mut buf = vec![0u8; n];
        let mut guard = self.inner.lock();
        match guard.read(&mut buf) {
            Ok(0) => Ok(None),
            Ok(k) => {
                buf.truncate(k);
                Ok(Some(buf))
            }
            Err(e) => Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                head: ":wat::io::read".into(),
                reason: format!("stdin read: {}", e)
            })),
        }
    }

    fn read_all(&self, span: Span) -> Result<Vec<u8>, RuntimeError> {
        use std::io::Read;
        let mut buf = Vec::new();
        let mut guard = self.inner.lock();
        guard.read_to_end(&mut buf).map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::read-all".into(),
            reason: format!("stdin read: {}", e)
        }))?;
        Ok(buf)
    }

    fn read_line(&self, span: Span) -> Result<Option<String>, RuntimeError> {
        use std::io::BufRead;
        let mut guard = self.inner.lock();
        let mut buf = String::new();
        match guard.read_line(&mut buf) {
            Ok(0) => Ok(None),
            Ok(_) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Ok(Some(buf))
            }
            Err(e) => Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                head: ":wat::io::read-line".into(),
                reason: format!("stdin read-line: {}", e)
            })),
        }
    }

    fn rewind(&self, _span: Span) -> Result<(), RuntimeError> {
        // Real stdin is not rewindable — this is a no-op per the trait
        // contract. If a test program calls rewind on real stdin it's
        // probably a portability bug, but the no-op matches Rust's
        // `Stdin::rewind` absence.
        Ok(())
    }

    /// Arc 170 — stdin joins the lock-step. `RealStdin` wraps `std::io::Stdin`,
    /// which is always fd 0; reporting `None` here (the old behavior) is what
    /// took stdin out of the substrate's poll-multiplex and left its read as
    /// a bare blocking `read(2)` — the one wait that could not observe a stop
    /// request. `Some(0)` lets `eval_ioreader_read_frame` (this module) poll
    /// `[0, SHUTDOWN_BROADCAST_READ_FD]` before every physical-line read,
    /// exactly as `channel/transfer.rs`'s `PipeFd` path already does.
    fn as_raw_fd_for_poll(&self) -> Option<i32> {
        Some(0)
    }
}

/// Wraps Rust's `std::io::Stdout`.
#[derive(Debug)]
pub struct RealStdout {
    pub(crate) inner: Arc<std::io::Stdout>,
}

impl RealStdout {
    pub fn new(inner: Arc<std::io::Stdout>) -> Self {
        Self { inner }
    }
}

impl WatWriter for RealStdout {
    fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.write(bytes).map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::write".into(),
            reason: format!("stdout write: {}", e)
        }))
    }

    fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.write_all(bytes).map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::write-all".into(),
            reason: format!("stdout write-all: {}", e)
        }))
    }

    fn flush(&self, span: Span) -> Result<(), RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.flush().map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::flush".into(),
            reason: format!("stdout flush: {}", e)
        }))
    }
}

/// Wraps Rust's `std::io::Stderr`.
#[derive(Debug)]
pub struct RealStderr {
    pub(crate) inner: Arc<std::io::Stderr>,
}

impl RealStderr {
    pub fn new(inner: Arc<std::io::Stderr>) -> Self {
        Self { inner }
    }
}

impl WatWriter for RealStderr {
    fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.write(bytes).map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::write".into(),
            reason: format!("stderr write: {}", e)
        }))
    }

    fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.write_all(bytes).map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::write-all".into(),
            reason: format!("stderr write-all: {}", e)
        }))
    }

    fn flush(&self, span: Span) -> Result<(), RuntimeError> {
        use std::io::Write;
        let mut guard = self.inner.lock();
        guard.flush().map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::flush".into(),
            reason: format!("stderr flush: {}", e)
        }))
    }
}

// ─── In-memory stand-ins (ThreadOwnedCell-backed; zero Mutex) ───────────

/// Read state for [`StringIoReader`] — backing bytes + current cursor.
#[derive(Debug)]
struct ReaderState {
    bytes: Vec<u8>,
    cursor: usize,
}

/// `:wat::io::IOReader` impl backed by an in-memory `Vec<u8>`. Pre-seed
/// from `from_bytes` or `from_string` at construction; subsequent
/// `read` / `read_line` / `read_all` / `rewind` ops mutate the cursor
/// under a `ThreadOwnedCell` — single-thread-owned; cross-thread use
/// panics with the owner-check error.
#[derive(Debug)]
pub struct StringIoReader {
    state: ThreadOwnedCell<ReaderState>,
}

impl StringIoReader {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            state: ThreadOwnedCell::new(ReaderState { bytes, cursor: 0 }),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self::from_bytes(s.into_bytes())
    }
}

impl WatReader for StringIoReader {
    fn read(&self, n: usize, span: Span) -> Result<Option<Vec<u8>>, RuntimeError> {
        self.state.with_mut(":wat::io::read", span, |s| {
            if s.cursor >= s.bytes.len() {
                return None;
            }
            let end = std::cmp::min(s.cursor + n, s.bytes.len());
            let out = s.bytes[s.cursor..end].to_vec();
            s.cursor = end;
            Some(out)
        })
    }

    fn read_all(&self, span: Span) -> Result<Vec<u8>, RuntimeError> {
        self.state.with_mut(":wat::io::read-all", span, |s| {
            let out = s.bytes[s.cursor..].to_vec();
            s.cursor = s.bytes.len();
            out
        })
    }

    fn read_line(&self, span: Span) -> Result<Option<String>, RuntimeError> {
        // Find next \n from cursor. Consume it. Decode as UTF-8.
        let bytes = self.state.with_mut(":wat::io::read-line", span.clone(), |s| {
            if s.cursor >= s.bytes.len() {
                return None;
            }
            // Search for newline.
            let rest = &s.bytes[s.cursor..];
            let line_end = rest.iter().position(|&b| b == b'\n');
            let (line_bytes, advance) = match line_end {
                Some(idx) => (&rest[..idx], idx + 1),
                None => (rest, rest.len()),
            };
            let bytes = line_bytes.to_vec();
            s.cursor += advance;
            Some(bytes)
        })?;
        match bytes {
            None => Ok(None),
            Some(mut b) => {
                // Strip trailing \r if the line was \r\n.
                if b.last() == Some(&b'\r') {
                    b.pop();
                }
                match String::from_utf8(b) {
                    Ok(s) => Ok(Some(s)),
                    Err(e) => Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                        head: ":wat::io::read-line".into(),
                        reason: format!("invalid UTF-8 in line: {}", e)
                    })),
                }
            }
        }
    }

    fn rewind(&self, span: Span) -> Result<(), RuntimeError> {
        self.state.with_mut(":wat::io::rewind", span, |s| {
            s.cursor = 0;
        })
    }
}

/// `:wat::io::IOWriter` impl backed by an in-memory `Vec<u8>`. Appends
/// on every write. `ThreadOwnedCell`-backed; single-thread-owned.
/// Readable via [`StringIoWriter::snapshot_bytes`] — intended for test
/// harnesses that invoke the writer, then capture what was written.
#[derive(Debug)]
pub struct StringIoWriter {
    buf: ThreadOwnedCell<Vec<u8>>,
}

impl Default for StringIoWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl StringIoWriter {
    pub fn new() -> Self {
        Self {
            buf: ThreadOwnedCell::new(Vec::new()),
        }
    }

    /// Clone the accumulated bytes. Owner-check enforced.
    pub fn snapshot_bytes(&self) -> Result<Vec<u8>, RuntimeError> {
        self.buf
            .with_ref(":wat::io::IOWriter::snapshot", |b| b.clone())
    }

    /// UTF-8 decode the accumulated bytes into a `String`. Returns
    /// `None` on invalid UTF-8. Owner-check enforced.
    pub fn snapshot_string(&self) -> Result<Option<String>, RuntimeError> {
        let bytes = self.snapshot_bytes()?;
        Ok(String::from_utf8(bytes).ok())
    }
}

impl WatWriter for StringIoWriter {
    fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError> {
        let n = bytes.len();
        self.buf.with_mut(":wat::io::write", span, |b| {
            b.extend_from_slice(bytes);
        })?;
        Ok(n)
    }

    fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError> {
        self.buf.with_mut(":wat::io::write-all", span, |b| {
            b.extend_from_slice(bytes);
        })
    }

    fn flush(&self, _span: Span) -> Result<(), RuntimeError> {
        // In-memory buffer — nothing to flush.
        Ok(())
    }

    fn snapshot(&self) -> Option<Vec<u8>> {
        // Owner-check enforced; returns None if called from the wrong
        // thread (same honest failure as all other ops).
        self.buf.with_ref(":wat::io::IOWriter/snapshot", |b| b.clone()).ok()
    }
}

// ─── Pipe-backed IOReader / IOWriter (arc 012 slice 1) ───────────────────
//
// fd-backed IO that bypasses Rust's `std::io::Read` / `Write` layers
// entirely. `PipeReader` / `PipeWriter` wrap an `OwnedFd` and call
// `libc::read(2)` / `write(2)` / `close(2)` directly.
//
// Why direct syscalls: `RealStdin` and friends wrap `std::io::Stdin`,
// which holds a reentrant Mutex internally. If arc 012's fork primitive
// ever inherited one of those locks held by a parent thread, the child
// would deadlock on any subsequent stdio call. Pipe-backed IO sidesteps
// the entire stdlib lock graph — nothing to inherit, nothing to
// deadlock on.
//
// Dual role: the `:wat::kernel::pipe` primitive produces these around
// a fresh `pipe2(O_CLOEXEC)` pair (parent-side pipe ends). The
// `:wat::kernel::spawn-process` primitive produces them
// around the child's dup2'd fd 0 / 1 / 2 via
// `from_owned_fd(OwnedFd::from_raw_fd(0))` etc. Same type, different
// owning fd.

use std::os::fd::{AsRawFd, IntoRawFd, OwnedFd};
use std::sync::atomic::{AtomicI32, Ordering};

/// `:wat::io::IOReader` backed by a raw fd. Wraps an `OwnedFd`;
/// `Drop` calls `close(2)` via `OwnedFd`'s stdlib impl. Read paths
/// call `libc::read(2)` directly — no `std::io::Read` detour, no
/// lock inheritance across fork.
#[derive(Debug)]
pub struct PipeReader {
    fd: OwnedFd,
}

impl PipeReader {
    /// Take ownership of an already-opened readable fd. Caller
    /// guarantees the fd is valid and readable (a pipe read-end or
    /// a redirected stdio fd). `Drop` will close it.
    pub fn from_owned_fd(fd: OwnedFd) -> Self {
        Self { fd }
    }
}

impl WatReader for PipeReader {
    fn read(&self, n: usize, span: Span) -> Result<Option<Vec<u8>>, RuntimeError> {
        let mut buf = vec![0u8; n];
        loop {
            let ret = unsafe {
                libc::read(self.fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, n)
            };
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::read".into(),
                    reason: format!("pipe read: {}", err)
                }));
            }
            if ret == 0 {
                return Ok(None);
            }
            buf.truncate(ret as usize);
            return Ok(Some(buf));
        }
    }

    fn read_all(&self, span: Span) -> Result<Vec<u8>, RuntimeError> {
        let mut out = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let ret = unsafe {
                libc::read(
                    self.fd.as_raw_fd(),
                    buf.as_mut_ptr() as *mut _,
                    buf.len(),
                )
            };
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::read-all".into(),
                    reason: format!("pipe read: {}", err)
                }));
            }
            if ret == 0 {
                return Ok(out);
            }
            out.extend_from_slice(&buf[..ret as usize]);
        }
    }

    fn read_line(&self, span: Span) -> Result<Option<String>, RuntimeError> {
        // Byte-at-a-time until `\n` or EOF. Pipes are kernel-buffered;
        // an extra read(2) per byte is cheap, and avoids maintaining a
        // user-level read-ahead buffer (which would need interior
        // mutability and undermine the plain `OwnedFd` shape).
        let mut bytes = Vec::new();
        let mut one = [0u8; 1];
        loop {
            let ret = unsafe {
                libc::read(self.fd.as_raw_fd(), one.as_mut_ptr() as *mut _, 1)
            };
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::read-line".into(),
                    reason: format!("pipe read: {}", err)
                }));
            }
            if ret == 0 {
                if bytes.is_empty() {
                    return Ok(None);
                }
                if bytes.last() == Some(&b'\r') {
                    bytes.pop();
                }
                return String::from_utf8(bytes)
                    .map(Some)
                    .map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                        head: ":wat::io::read-line".into(),
                        reason: format!("invalid UTF-8 in line: {}", e)
                    }));
            }
            if one[0] == b'\n' {
                if bytes.last() == Some(&b'\r') {
                    bytes.pop();
                }
                return String::from_utf8(bytes)
                    .map(Some)
                    .map_err(|e| RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                        head: ":wat::io::read-line".into(),
                        reason: format!("invalid UTF-8 in line: {}", e)
                    }));
            }
            bytes.push(one[0]);
        }
    }

    fn rewind(&self, span: Span) -> Result<(), RuntimeError> {
        Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
            head: ":wat::io::rewind".into(),
            reason: "pipe fds are not rewindable".into()
        }))
    }

    fn as_raw_fd_for_poll(&self) -> Option<i32> {
        Some(self.fd.as_raw_fd())
    }
}

/// `:wat::io::IOWriter` backed by a raw fd. The fd lives in an
/// `AtomicI32` so explicit `close()` can release it before the Arc
/// count reaches zero — needed when the writer Arc is held by a
/// struct (e.g., `:wat::kernel::Process.stdin`) and the caller
/// wants the peer reader to see EOF mid-program. Lock-free; one
/// atomic swap per close.
///
/// Sentinel `-1` means closed; reads + writes against the closed
/// fd return errors.
#[derive(Debug)]
pub struct PipeWriter {
    fd: AtomicI32,
}

impl PipeWriter {
    /// Take ownership of an already-opened writable fd. Caller
    /// guarantees the fd is valid and writable. We strip the
    /// `OwnedFd` wrapper because the AtomicI32 is now responsible
    /// for the fd's lifetime — Drop calls `close(2)` ourselves.
    pub fn from_owned_fd(fd: OwnedFd) -> Self {
        Self {
            fd: AtomicI32::new(fd.into_raw_fd()),
        }
    }
}

impl Drop for PipeWriter {
    /// Idempotent. If the fd is still live, swap to -1 and
    /// `close(2)` the original; if already closed, no-op.
    fn drop(&mut self) {
        let raw = self.fd.swap(-1, Ordering::SeqCst);
        if raw >= 0 {
            unsafe {
                libc::close(raw);
            }
        }
    }
}

impl WatWriter for PipeWriter {
    fn write(&self, bytes: &[u8], span: Span) -> Result<usize, RuntimeError> {
        loop {
            let raw = self.fd.load(Ordering::SeqCst);
            if raw < 0 {
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::write".into(),
                    reason: "pipe write: writer is closed".into()
                }));
            }

            // Arc 170 closure #5 — the writer joins the lock-step. Poll
            // `[fd → POLLOUT, SHUTDOWN_BROADCAST_READ_FD → POLLIN|POLLHUP]`
            // before every write attempt, exactly as `channel/transfer.rs`'s
            // `read_one_line` already does for the read side (shutdown wins
            // ties). When the broadcast fd hasn't been initialized (-1: test
            // bypass / pre-bootstrap), skip straight to the blocking write
            // below — today's un-multiplexed path, unchanged. This is the
            // StringIoWriter/no-broadcast fallback the brief requires; it
            // applies here via broadcast_fd rather than via
            // `as_raw_fd_for_poll()` because PipeWriter always has an fd
            // once open (checked above) — `as_raw_fd_for_poll()` reporting
            // `None` is exactly the "writer is closed" case already handled.
            let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(Ordering::SeqCst);
            if broadcast_fd >= 0 {
                loop {
                    let mut fds = [
                        libc::pollfd { fd: raw, events: libc::POLLOUT, revents: 0 },
                        libc::pollfd {
                            fd: broadcast_fd,
                            // POLLIN: the shutdown-worker's wake byte. POLLHUP: the
                            // broadcast write-end closing after — either wakes us.
                            events: libc::POLLIN | libc::POLLHUP,
                            revents: 0,
                        },
                    ];
                    let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
                    if n < 0 {
                        // EINTR re-polls; never a blind retry (constraint 3 —
                        // whether EINTR ever reaches here under this repo's
                        // signal handlers is deliberately unresolved; a
                        // poll-first loop is correct either way).
                        let err = std::io::Error::last_os_error();
                        if err.kind() == std::io::ErrorKind::Interrupted {
                            continue;
                        }
                        break; // non-EINTR poll error — proceed to write, let write(2) surface it
                    }
                    if n == 0 {
                        // timeout=-1 should never produce n=0; defensively retry.
                        continue;
                    }
                    // ⚠ THE TIE-BREAK IS THE OPPOSITE OF THE READER'S, DELIBERATELY.
                    //
                    // `channel/transfer.rs`'s reader gives shutdown the tie ("shutdown wins
                    // ties") and that is right THERE: a read racing a stop has nothing left to
                    // read, so preferring the stop loses nothing.
                    //
                    // A WRITE is not symmetric. A stop does not mean "stop writing" — it means
                    // "do not block forever". A process that is stopping still has to be able to
                    // say WHY: `eprintln` is the dying declaration (R51 TYPO TANGO — strict EDN
                    // out on stderr, then terminate), and the stdio services flush on the way
                    // down. Giving shutdown the tie here made a stopped process unable to utter
                    // its last words, in the arc whose law is that nothing hides a failure.
                    //
                    // MEASURED: preferring shutdown regressed
                    // `wat_cli::sigterm_reaches_a_program_blocked_on_stdin` from exit 0 to exit 1
                    // — a program that correctly observed a stop in its READ then failed on the
                    // write during teardown. Proven mine by a stash differential (passes without
                    // this file's change, fails with it).
                    //
                    // So: if the fd is writable NOW, WRITE — the write cannot block, and there is
                    // no reason to abandon it. Surface the stop only when the write WOULD have
                    // blocked, which is the entire defect this poll exists to fix.
                    if fds[0].revents != 0 {
                        break; // writable — proceed, stop or no stop
                    }
                    if fds[1].revents != 0 {
                        // Not writable AND a stop is pending → this write would block
                        // indefinitely. That is the hang; surface it as a named value.
                        return Err(RuntimeError::new(span, RuntimeErrorKind::WriteStopped));
                    }
                }
            }

            let ret = unsafe {
                libc::write(raw, bytes.as_ptr() as *const _, bytes.len())
            };
            if ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::Interrupted {
                    // Re-poll before retrying (constraint 3) rather than
                    // blind-retrying the write directly — loops back to the
                    // top, which re-checks closed state and re-polls.
                    continue;
                }
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::write".into(),
                    reason: format!("pipe write: {}", err)
                }));
            }
            return Ok(ret as usize);
        }
    }

    fn write_all(&self, bytes: &[u8], span: Span) -> Result<(), RuntimeError> {
        let mut remaining = bytes;
        while !remaining.is_empty() {
            let n = self.write(remaining, span.clone())?;
            if n == 0 {
                return Err(RuntimeError::new(span, RuntimeErrorKind::MalformedForm {
                    head: ":wat::io::write-all".into(),
                    reason: "pipe write returned 0 bytes".into()
                }));
            }
            remaining = &remaining[n..];
        }
        Ok(())
    }

    fn flush(&self, _span: Span) -> Result<(), RuntimeError> {
        // Pipes have no user-level buffer. Kernel-buffered bytes
        // become readable to the peer as soon as write(2) returns.
        Ok(())
    }

    fn close(&self, _span: Span) -> Result<(), RuntimeError> {
        let raw = self.fd.swap(-1, Ordering::SeqCst);
        if raw >= 0 {
            // Errors from close(2) are advisory (typically EINTR or
            // EIO from a previously-failed write). Don't surface;
            // the swap already marked the writer closed, and the
            // caller's interest is just "release the fd."
            unsafe {
                libc::close(raw);
            }
        }
        Ok(())
    }

    /// The raw fd this writer backs (arc 170 stdio-as-defservice bootstrap seeds the primed services
    /// with fd numbers). `-1` (closed) reports `None`. Mirrors the PipeReader override (io.rs).
    fn as_raw_fd_for_poll(&self) -> Option<i32> {
        let raw = self.fd.load(Ordering::SeqCst);
        if raw >= 0 { Some(raw) } else { None }
    }
}

// ─── Primitive handlers ──────────────────────────────────────────────────
//
// These are invoked from `runtime::eval`'s dispatch match on the head
// keyword; the runtime arm is a one-line call into here.

fn arity(op: &str, args: &[WatAST], n: usize, list_span: &Span) -> Result<(), RuntimeError> {
    if args.len() != n {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: n,
            got: args.len()
        }));
    }
    Ok(())
}

fn expect_reader(op: &str, tv: TrackedValue, span: Span) -> Result<Arc<dyn WatReader>, RuntimeError> {
    match tv.value_owned() {
        Value::io__IOReader(r) => Ok(r),
        other => Err(RuntimeError::new(span, RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::io::IOReader",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn expect_writer(op: &str, tv: TrackedValue, span: Span) -> Result<Arc<dyn WatWriter>, RuntimeError> {
    match tv.value_owned() {
        Value::io__IOWriter(w) => Ok(w),
        other => Err(RuntimeError::new(span, RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::io::IOWriter",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn expect_i64(op: &str, tv: TrackedValue, span: Span) -> Result<i64, RuntimeError> {
    match tv.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(span, RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "i64",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn expect_string(op: &str, tv: TrackedValue, span: Span) -> Result<Arc<String>, RuntimeError> {
    match tv.value_owned() {
        Value::String(s) => Ok(s),
        other => Err(RuntimeError::new(span, RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn expect_vec_u8(op: &str, tv: TrackedValue, span: Span) -> Result<Vec<u8>, RuntimeError> {
    match tv.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::u8(b) => out.push(*b),
                    other => {
                        // arc 138: no span — Vec element iteration; per-element WatAST span unavailable; use form span
                        return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "u8",
                            got: Box::new(crate::runtime::ValueSnapshot::of(other))
                        }));
                    }
                }
            }
            Ok(out)
        }
        other => Err(RuntimeError::new(span, RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "Vec<u8>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        })),
    }
}

fn bytes_to_vec_u8_value(bytes: Vec<u8>) -> Value {
    Value::Vec(Arc::new(bytes.into_iter().map(Value::u8).collect()))
}

// ─── IOReader construction ──────────────────────────────────────────────

/// `(:wat::io::IOReader/from-bytes <Vec<u8>>)` → `:wat::io::IOReader`.
pub fn eval_ioreader_from_bytes(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/from-bytes";
    arity(op, args, 1, list_span)?;
    let bytes = expect_vec_u8(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let reader: Arc<dyn WatReader> = Arc::new(StringIoReader::from_bytes(bytes));
    Ok(Value::io__IOReader(reader))
}

/// `(:wat::io::IOReader/from-string <String>)` → `:wat::io::IOReader`.
pub fn eval_ioreader_from_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/from-string";
    arity(op, args, 1, list_span)?;
    let s = expect_string(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let reader: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string((*s).clone()));
    Ok(Value::io__IOReader(reader))
}

// ─── IOReader ops ────────────────────────────────────────────────────────

/// `(:wat::io::IOReader/read <reader> <i64>)` → `(:Option :- [(Vec :- [u8])])`.
pub fn eval_ioreader_read(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/read";
    arity(op, args, 2, list_span)?;
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let n = expect_i64(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    if n < 0 {
        return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("negative byte count: {}", n)
        }));
    }
    let result = reader.read(n as usize, list_span.clone())?;
    Ok(Value::Option(Arc::new(result.map(bytes_to_vec_u8_value))))
}

/// `(:wat::io::IOReader/read-all <reader>)` → `(:wat::core::Vector :- [u8])`.
pub fn eval_ioreader_read_all(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/read-all";
    arity(op, args, 1, list_span)?;
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let bytes = reader.read_all(list_span.clone())?;
    Ok(bytes_to_vec_u8_value(bytes))
}

/// `(:wat::io::IOReader/read-all-string <reader>)` → `:wat::core::String`. Reads the
/// reader to EOF and UTF-8-decodes the bytes — byte-faithful (no line-splitting, no
/// reconstruction). The read mirror of `IOWriter/to-string`; the rung `read-file` rides.
/// A non-UTF-8 stream is an environment error worth halting on (panic-vs-Option).
pub fn eval_ioreader_read_all_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/read-all-string";
    arity(op, args, 1, list_span)?;
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let bytes = reader.read_all(list_span.clone())?;
    let s = String::from_utf8(bytes).unwrap_or_else(|e| panic!("{op}: invalid UTF-8: {e}"));
    Ok(Value::String(Arc::new(s)))
}

/// `(:wat::io::IOReader/read-line <reader>)` → `(:Option :- [String])`.
pub fn eval_ioreader_read_line(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/read-line";
    arity(op, args, 1, list_span)?;
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let line = reader.read_line(list_span.clone())?;
    Ok(Value::Option(Arc::new(
        line.map(|s| Value::String(Arc::new(s))),
    )))
}

/// `(:wat::io::IOReader/read-frame <reader>)` → `:wat::io::IOReader::ReadFrameOutcome`.
/// `(:wat::io::IOReader/read-frame <reader> <max-bytes :i64>)` → `:wat::io::IOReader::ReadFrameOutcome`.
///
/// Accumulates physical lines from `reader` (via `read_line`) until the
/// buffer forms a complete EDN value, then returns
/// `ReadFrameOutcome::Frame(trimmed-frame-string)`. Returns
/// `ReadFrameOutcome::Eof` on clean EOF (no bytes consumed), and
/// `ReadFrameOutcome::Stopped` if a process-wide stop was requested while
/// waiting. Surfaces a `RuntimeError` if the frame is `Truncated` (EOF
/// mid-frame) or `Malformed` (syntax error in the accumulated buffer).
///
/// Arc 170 — before every physical-line read, polls `reader`'s fd (when
/// `as_raw_fd_for_poll` reports one — true for `RealStdin` as of this arc,
/// and for `PipeReader`) against `SHUTDOWN_BROADCAST_READ_FD`, exactly as
/// `channel/transfer.rs`'s `PipeFd` recv path already does. Shutdown wins
/// ties. This is the fix that lets a program blocked in `read-frame` on
/// real stdin observe SIGTERM instead of pinning the process alive until
/// stdin EOFs.
///
/// `(Option :- [String])` could not carry the third ("a stop was requested, and
/// nothing is wrong with the stream") outcome, so this verb's return type
/// changed from `(Option :- [String])` to the dedicated `ReadFrameOutcome` enum —
/// see its scheme in `src/check.rs`.
///
/// Optional second argument: explicit max-bytes cap (i64). When absent the
/// cap defaults to `DEFAULT_MAX_FRAME_BYTES` (512 KiB). The 2-arg form is
/// used by `StdInService/handle` when `readln'` passes a caller-supplied cap.
///
/// Consumes `crate::edn_shim::read_framed_edn` — the live wire-protocol
/// accumulator.
pub fn eval_ioreader_read_frame(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/read-frame";
    if args.is_empty() || args.len() > 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "expected 1 or 2 args (reader [max-bytes]); got {}",
                    args.len()
                ),
            }));
    }
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    use crate::edn_shim::{read_framed_edn, FramedRead, DEFAULT_MAX_FRAME_BYTES};
    let cap: usize = if args.len() == 2 {
        match eval(&args[1], env, sym)?.value_owned() {
            Value::i64(n) if n > 0 => n as usize,
            Value::i64(n) => {
                return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::MalformedForm {
                        head: op.into(),
                        reason: format!("max-bytes must be positive; got {}", n),
                    }));
            }
            other => {
                return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                        op: op.into(),
                        expected: "i64 max-bytes",
                        got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                    }));
            }
        }
    } else {
        DEFAULT_MAX_FRAME_BYTES
    };
    // Arc 170 — poll the reader's fd (if it has one) against the shutdown
    // broadcast fd before every physical-line read, exactly as
    // `channel/transfer.rs`'s `PipeFd` recv path (`read_one_line`) already
    // does for the peer-to-peer wire. `RealStdin` now reports `Some(0)`
    // (see its `as_raw_fd_for_poll` impl, this module), so this is the
    // read that brings stdin into the lock-step.
    let pipe_fd_opt = reader.as_raw_fd_for_poll();
    let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
        std::sync::atomic::Ordering::SeqCst,
    );
    use crate::edn_shim::LineRead;
    let next_line = |span: Span| -> Result<LineRead, RuntimeError> {
        if let (Some(pfd), true) = (pipe_fd_opt, broadcast_fd >= 0) {
            loop {
                let mut fds = [
                    libc::pollfd {
                        fd: pfd,
                        events: libc::POLLIN | libc::POLLHUP,
                        revents: 0,
                    },
                    libc::pollfd {
                        fd: broadcast_fd,
                        // Arc 170 Phase 1 — broadcast means WAKE (POLLIN, a
                        // written byte) as well as SEVER (POLLHUP, the drop
                        // that still immediately follows the write today).
                        events: libc::POLLIN | libc::POLLHUP,
                        revents: 0,
                    },
                ];
                let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
                if n < 0 {
                    // EINTR → retry; other errors fall through to read_line.
                    let err = std::io::Error::last_os_error();
                    if err.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    break; // non-EINTR error — proceed to read_line
                }
                if n == 0 {
                    // timeout=-1 should never produce n=0; defensively retry.
                    continue;
                }
                // Shutdown wins ties per Slice B discipline (matches
                // channel/transfer.rs's read_one_line exactly).
                if fds[1].revents != 0 {
                    return Ok(LineRead::Shutdown);
                }
                if fds[0].revents != 0 {
                    break;
                }
            }
        }
        match reader.read_line(span)? {
            Some(line) => Ok(LineRead::Line(line)),
            None => Ok(LineRead::Eof),
        }
    };
    // This match is the ONE site where the Rust-side `FramedRead` vocabulary crosses
    // into the wat-visible `:wat::io::IOReader::ReadFrameOutcome` vocabulary, and the
    // names deliberately diverge here: `FramedRead::Shutdown` keeps Rust's uniform
    // `shutdown` naming (it mirrors `channel/transfer.rs`'s `LineResult::Shutdown`,
    // the file this poll was copied from), while the wat variant is `Stopped` — wat
    // already has `(:wat::kernel::stopped?)` for this fact, and nothing is shutting
    // down when this is produced, a stop was merely requested. Ruled by the arc-170
    // intueri cast (2026-07-28): the rename boundary IS the Rust/wat boundary, which
    // is exactly where the audience — and so the vocabulary — is allowed to change.
    match read_framed_edn(next_line, list_span.clone(), cap)? {
        FramedRead::Frame(buf) => {
            // Trim the trailing newline that read_framed_edn appends.
            let s = buf.trim_end_matches('\n').to_string();
            Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
                type_path: ":wat::io::IOReader::ReadFrameOutcome".into(),
                variant_name: "Frame".into(),
                names: crate::runtime::builtin_enum_variant_names(":wat::io::IOReader::ReadFrameOutcome", "Frame"),
                fields: vec![Value::String(Arc::new(s))],
            })))
        }
        FramedRead::Eof => Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
            type_path: ":wat::io::IOReader::ReadFrameOutcome".into(),
            variant_name: "Eof".into(),
            names: crate::runtime::no_field_names(),
            fields: vec![],
        }))),
        // Arc 170 — the whole point: a stop request is NOT an EOF and NOT an
        // error, so it gets its own outcome here rather than being folded
        // into `Eof` (which would report a stop as a clean writer-closed
        // and is the precise defect this brief exists to remove) or an
        // `Err` (which would report it as a malfunction it isn't).
        FramedRead::Shutdown => Ok(Value::Enum(Arc::new(crate::runtime::EnumValue {
            type_path: ":wat::io::IOReader::ReadFrameOutcome".into(),
            variant_name: "Stopped".into(),
            names: crate::runtime::no_field_names(),
            fields: vec![],
        }))),
        FramedRead::Truncated(partial) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "EOF arrived mid-frame (truncated): {:?}",
                    partial
                ),
            })),
        FramedRead::Malformed(msg) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!("malformed EDN frame: {}", msg),
            })),
        FramedRead::TooLarge(n) => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!(
                    "EDN frame exceeded {} bytes without completing — message too large or never terminated",
                    n
                ),
            })),
    }
}

/// `(:wat::io::IOReader/rewind <reader>)` → `:()`.
pub fn eval_ioreader_rewind(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOReader/rewind";
    arity(op, args, 1, list_span)?;
    let reader = expect_reader(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    reader.rewind(list_span.clone())?;
    Ok(Value::Unit)
}

// ─── IOWriter construction + snapshot ───────────────────────────────────

/// `(:wat::io::IOWriter/new)` → `:wat::io::IOWriter` (empty).
pub fn eval_iowriter_new(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/new";
    arity(op, args, 0, list_span)?;
    let writer: Arc<dyn WatWriter> = Arc::new(StringIoWriter::new());
    Ok(Value::io__IOWriter(writer))
}

/// `(:wat::io::IOWriter/open-file path)` → `:wat::io::IOWriter`. Opens
/// (or creates+truncates) a regular file at `path` for writing and
/// returns a file-backed IOWriter. Each `write` call goes through
/// `libc::write(2)` on the underlying fd; `Drop` closes via OwnedFd.
///
/// Used by long-running wat programs that manage their own per-run
/// log files (e.g., trader programs writing `runs/<id>.out` and
/// `runs/<id>.err` instead of inheriting the parent process's
/// stdout/stderr).
///
/// Panics on open errors via the panic-vs-Option discipline (memory
/// `feedback_shim_panic_vs_option`): bad path / permission / disk-full
/// at construction-time is an environment error worth halting on.
pub fn eval_iowriter_open_file(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    use std::os::fd::OwnedFd;
    let op = ":wat::io::IOWriter/open-file";
    arity(op, args, 1, list_span)?;
    let path = match crate::runtime::eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap_or_else(|e| panic!(":wat::io::IOWriter/open-file {path:?}: {e}"));
    let fd: OwnedFd = file.into();
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(fd));
    Ok(Value::io__IOWriter(writer))
}

/// `(:wat::io::IOReader/open-file path)` → `:wat::io::IOReader`. Opens a
/// regular file at `path` for reading and returns a file-backed IOReader.
/// Each `read`/`read-all` goes through `libc::read(2)` on the underlying
/// fd; `Drop` closes via OwnedFd. The read-side mirror of
/// `IOWriter/open-file` — completes the explicit rung of the file ladder.
///
/// Panics on open errors via the panic-vs-Option discipline (memory
/// `feedback_shim_panic_vs_option`): bad path / permission at
/// construction-time is an environment error worth halting on.
pub fn eval_ioreader_open_file(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    use std::os::fd::OwnedFd;
    let op = ":wat::io::IOReader/open-file";
    arity(op, args, 1, list_span)?;
    let path = match crate::runtime::eval(&args[0], env, sym)?.value_owned() {
        Value::String(s) => (*s).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::String",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            }));
        }
    };
    let file = std::fs::OpenOptions::new()
        .read(true)
        .open(&path)
        .unwrap_or_else(|e| panic!(":wat::io::IOReader/open-file {path:?}: {e}"));
    let fd: OwnedFd = file.into();
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(fd));
    Ok(Value::io__IOReader(reader))
}

/// `(:wat::io::IOWriter/from-fd fd)` → `:wat::io::IOWriter`. Arc 170 stdio-as-defservice.
///
/// **DUP-then-own (load-bearing).** `dup(2)` the caller's raw fd first, then wrap the DUP in an
/// `OwnedFd`-backed [`PipeWriter`]. The returned writer owns ONLY the dup: its `Drop` closes the
/// dup, NEVER the process's original fd (so a primed StdOut/StdErr service holding the writer in
/// `:ephemeral` can shut down without closing the real fd 1/2). Mirror of `eval_kernel_pipe`'s
/// `from_owned_fd` ownership (io.rs) and `process/verbs.rs`'s stdio-fd wrapping.
///
/// **Restricted to `:wat::kernel::` callers** — forging an `IOWriter` from an arbitrary raw fd is a
/// capability; only kernel-internal wat (the primed stdio defservices' generated `::init`) may call
/// it. The fd itself is a pure `i64` (it rides `Admin::Init` clean); the impure handle is BORN here,
/// inside the kernel init body — never passed as an init param (which would trip the Pure-`Admin`
/// containment wall, arc 293.W).
#[restricted_to(":wat::io::IOWriter/from-fd", ":wat::kernel::")]
pub fn eval_iowriter_from_fd(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    use std::os::fd::{FromRawFd, OwnedFd};
    let op = ":wat::io::IOWriter/from-fd";
    arity(op, args, 1, list_span)?;
    let fd = match crate::runtime::eval(&args[0], env, sym)?.value_owned() {
        Value::i64(n) => n,
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::i64",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
            }));
        }
    };
    // dup(2): the service owns a PRIVATE copy of the fd; dropping the writer closes the dup only,
    // never the caller's real fd 0/1/2.
    let dup_fd = unsafe { libc::dup(fd as libc::c_int) };
    if dup_fd < 0 {
        let e = std::io::Error::last_os_error();
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("dup(2) on fd {fd} failed: {e}"),
        }));
    }
    // SAFETY: dup(2) returned a fresh, owned fd; OwnedFd takes ownership and Drop calls close(2).
    let owned = unsafe { OwnedFd::from_raw_fd(dup_fd) };
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(owned));
    Ok(Value::io__IOWriter(writer))
}

/// `(:wat::io::IOReader/from-fd fd)` → `:wat::io::IOReader`. Arc 170 stdio-as-defservice. The read
/// mirror of [`eval_iowriter_from_fd`]: `dup(2)`-then-own, wrap the DUP in an `OwnedFd`-backed
/// [`PipeReader`]. Dropping the reader closes the dup only, never the process's real fd 0.
/// Restricted to `:wat::kernel::` callers (the primed StdIn defservice's generated `::init`).
#[restricted_to(":wat::io::IOReader/from-fd", ":wat::kernel::")]
pub fn eval_ioreader_from_fd(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    use std::os::fd::{FromRawFd, OwnedFd};
    let op = ":wat::io::IOReader/from-fd";
    arity(op, args, 1, list_span)?;
    let fd = match crate::runtime::eval(&args[0], env, sym)?.value_owned() {
        Value::i64(n) => n,
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::i64",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
            }));
        }
    };
    let dup_fd = unsafe { libc::dup(fd as libc::c_int) };
    if dup_fd < 0 {
        let e = std::io::Error::last_os_error();
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("dup(2) on fd {fd} failed: {e}"),
        }));
    }
    // SAFETY: dup(2) returned a fresh, owned fd; OwnedFd takes ownership and Drop calls close(2).
    let owned = unsafe { OwnedFd::from_raw_fd(dup_fd) };
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(owned));
    Ok(Value::io__IOReader(reader))
}

/// `(:wat::io::IOWriter/to-bytes <writer>)` → `(:wat::core::Vector :- [u8])`. Clones the
/// accumulated buffer. Only valid for `StringIoWriter` — real stdio
/// doesn't snapshot (returns MalformedForm).
pub fn eval_iowriter_to_bytes(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/to-bytes";
    arity(op, args, 1, list_span)?;
    let writer_value = eval(&args[0], env, sym)?;
    let writer = expect_writer(op, writer_value, args[0].span().clone())?;
    let bytes = snapshot_writer(op, &writer)?;
    Ok(bytes_to_vec_u8_value(bytes))
}

/// `(:wat::io::IOWriter/to-string <writer>)` → `(:Option :- [String])`. UTF-8
/// decode of the accumulated buffer; `:None` if not valid UTF-8.
pub fn eval_iowriter_to_string(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/to-string";
    arity(op, args, 1, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let bytes = snapshot_writer(op, &writer)?;
    let decoded = String::from_utf8(bytes).ok();
    Ok(Value::Option(Arc::new(
        decoded.map(|s| Value::String(Arc::new(s))),
    )))
}

/// Helper: snapshot a writer's accumulated bytes. Only meaningful for
/// `StringIoWriter`; real stdio refuses.
fn snapshot_writer(
    op: &str,
    writer: &Arc<dyn WatWriter>,
) -> Result<Vec<u8>, RuntimeError> {
    // Downcast via a capability method: StringIoWriter supports
    // snapshotting; real-stdio writers don't. We expose it via the
    // trait itself — there's no need to downcast at dispatch time
    // if every impl answers "can I be snapshotted?" honestly.
    //
    // Simplest: extend WatWriter with an optional `snapshot` method
    // that defaults to returning NotSupported. StringIoWriter
    // overrides.
    // arc 138: no span — helper receives evaluated Arc<dyn WatWriter>, no WatAST; Value-only context
    writer.snapshot().ok_or_else(|| RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::MalformedForm {
        head: op.into(),
        reason: "writer does not support snapshot (only StringIoWriter does)"
            .into()
    }))
}

// ─── IOWriter ops ────────────────────────────────────────────────────────

/// `(:wat::io::IOWriter/write <writer> <Vec<u8>>)` → `:i64` (bytes written).
pub fn eval_iowriter_write(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/write";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let bytes = expect_vec_u8(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    let n = writer.write(&bytes, list_span.clone())?;
    Ok(Value::i64(n as i64))
}

/// `(:wat::io::IOWriter/write-all <writer> <Vec<u8>>)` → `:()`.
pub fn eval_iowriter_write_all(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/write-all";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let bytes = expect_vec_u8(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    writer.write_all(&bytes, list_span.clone())?;
    Ok(Value::Unit)
}

/// `(:wat::io::IOWriter/write-string <writer> <String>)` → `:i64`
/// (bytes written, no trailing newline). UTF-8 encodes the String and
/// writes its bytes via `write_all`. Companion to `writeln` — same
/// shape but without the implicit `\n`. Matches the semantics of the
/// pre-arc-008 `:wat::io::write` on real Stdout/Stderr.
pub fn eval_iowriter_write_string(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/write-string";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let s = expect_string(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    let bytes = s.as_bytes();
    let n = bytes.len();
    writer.write_all(bytes, list_span.clone())?;
    Ok(Value::i64(n as i64))
}

/// `(:wat::io::IOWriter/print <writer> <String>)` → `:()`. Unit-
/// returning convenience over `write-string`; discards the byte
/// count. Use when you want "write this and move on" — matches
/// Ruby's `$stdout.print`.
pub fn eval_iowriter_print(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/print";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let s = expect_string(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    writer.write_all(s.as_bytes(), list_span.clone())?;
    Ok(Value::Unit)
}

/// `(:wat::io::IOWriter/println <writer> <String>)` → `:()`. Unit-
/// returning convenience over `writeln`; writes `s` + `\n` and
/// discards the byte count. Matches Ruby's `$stdout.puts`.
pub fn eval_iowriter_println(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/println";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let s = expect_string(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(b'\n');
    writer.write_all(&bytes, list_span.clone())?;
    Ok(Value::Unit)
}

/// `(:wat::io::IOWriter/writeln <writer> <String>)` → `:i64` (bytes
/// written, including the trailing `\n`).
pub fn eval_iowriter_writeln(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/writeln";
    arity(op, args, 2, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let s = expect_string(op, eval(&args[1], env, sym)?, args[1].span().clone())?;
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(b'\n');
    let n = bytes.len();
    writer.write_all(&bytes, list_span.clone())?;
    Ok(Value::i64(n as i64))
}

/// `(:wat::io::IOWriter/flush <writer>)` → `:()`.
pub fn eval_iowriter_flush(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/flush";
    arity(op, args, 1, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    writer.flush(list_span.clone())?;
    Ok(Value::Unit)
}

/// `(:wat::io::IOWriter/close <writer>)` → `:()`. Idempotent.
///
/// For pipe-backed writers, releases the fd immediately — peer
/// readers see EOF on next read. Needed because the writer Arc may
/// be held by an enclosing struct (e.g.,
/// `:wat::kernel::Process.stdin`) that outlives the write phase;
/// without explicit close, the kernel pipe stays open until the
/// struct drops. For non-pipe backings (StringIoWriter, RealStdout,
/// RealStderr) close is a no-op — closing real OS stdio would
/// break the parent process. Subsequent writes against a closed
/// pipe writer return an error.
///
/// Arc 103b enabler. The wat-level `run-sandboxed` helper writes
/// each pre-seeded stdin line to `proc.stdin`, calls close to
/// signal EOF, then drains `proc.stdout` / `proc.stderr`.
pub fn eval_iowriter_close(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::IOWriter/close";
    arity(op, args, 1, list_span)?;
    let writer = expect_writer(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    writer.close(list_span.clone())?;
    Ok(Value::Unit)
}

// ─── :wat::kernel::pipe (arc 012 slice 1b) ───────────────────────────────

/// `(:wat::kernel::pipe)` → `:(wat::io::IOWriter, wat::io::IOReader)`.
///
/// Creates a fresh Unix pipe via `libc::pipe2(2)` with `O_CLOEXEC`. The write
/// end comes first in the returned tuple (you write to produce, read to consume
/// — same order a human says "producer then consumer"). Both ends are
/// `PipeWriter` / `PipeReader` over an `OwnedFd`; `Drop` closes.
///
/// Arc 012 slice 1. Standalone useful (any IPC pattern that wants a
/// byte stream between wat threads or into a child process);
/// load-bearing for `:wat::kernel::spawn-process` which allocates
/// three pipes per fork call.
pub fn eval_kernel_pipe(args: &[WatAST], list_span: &Span) -> Result<Value, RuntimeError> {
    use std::os::fd::FromRawFd;
    let op = ":wat::kernel::pipe";
    arity(op, args, 0, list_span)?;
    let mut fds = [0i32; 2];
    // pipe2(O_CLOEXEC): atomic CLOEXEC at creation. Belt for any future exec
    // path; in fork-without-exec the flag doesn't fire (no exec to clear it).
    let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("pipe2(2) syscall failed: {}", err)
        }));
    }
    // SAFETY: libc::pipe2 returned 0, so fds[0] (read) and fds[1]
    // (write) are freshly-opened fds we now own; wrapping each in
    // OwnedFd transfers that ownership. `Drop` will call close(2).
    let reader_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let writer_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(writer_fd));
    let reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(reader_fd));
    Ok(Value::Tuple(Arc::new(vec![
        Value::io__IOWriter(writer),
        Value::io__IOReader(reader),
    ])))
}

// ─── Scope-bound temp file + temp dir (arc 093 slice 1e) ─────────────────
//
// Wraps Rust's `tempfile` crate as substrate-internal primitives.
// Both types create a fresh file/dir under `std::env::temp_dir()`
// at construction; Drop auto-unlinks (or `remove_dir_all`s) when
// the wat value's Arc-count reaches zero — i.e., when the binding
// goes out of `let` scope and no one else holds the cell.
//
// Caller idiom (tests, ad-hoc scripts):
//
// ```scheme
// (:wat::core::let
//   (((tf :wat::io::TempFile) (:wat::io::TempFile/new))
//    ((path :String) (:wat::io::TempFile/path tf))
//    ;; ... use path ...
//    ((_ :()) (:do-stuff path)))
//   ;; Drop fires here on exit — the file is gone.
//   ())
// ```
//
// `TempFile` creates a regular empty file with a random name;
// `TempDir` creates an empty directory (recursive cleanup at
// Drop). Use TempFile for "I want a path to write to"; use
// TempDir for "I need a scratch directory for multiple files."

use tempfile::{NamedTempFile, TempDir};

/// `:wat::io::TempFile` — auto-deleting temp file. Wraps
/// `tempfile::NamedTempFile`. Drop unlinks the file.
pub struct WatTempFile {
    /// `Option` so `Drop` can `take()` and let `tempfile`'s own
    /// Drop run. Always `Some` while the value is alive.
    pub inner: Option<NamedTempFile>,
}

impl WatTempFile {
    pub fn new(span: &Span) -> Result<Self, RuntimeError> {
        match NamedTempFile::new() {
            Ok(f) => Ok(Self { inner: Some(f) }),
            // The wat call site's span (threaded from `eval_io_temp_file_new`) locates this OS
            // error at the user's `(:wat::io::TempFile/new)`, not at a coarse `rust_caller_span!()`.
            Err(e) => Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: ":wat::io::TempFile/new".into(),
                reason: format!("create temp file: {e}")
            })),
        }
    }

    pub fn path(&self) -> Result<String, RuntimeError> {
        match &self.inner {
            Some(f) => Ok(f.path().display().to_string()),
            // arc 138: no span — runtime invariant; no WatAST at this call depth
            None => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::MalformedForm {
                head: ":wat::io::TempFile/path".into(),
                reason: "TempFile already dropped".into()
            })),
        }
    }
}

/// `:wat::io::TempDir` — auto-deleting temp directory. Wraps
/// `tempfile::TempDir`. Drop runs `remove_dir_all`.
pub struct WatTempDir {
    pub inner: Option<TempDir>,
}

impl WatTempDir {
    pub fn new(span: &Span) -> Result<Self, RuntimeError> {
        match TempDir::new() {
            Ok(d) => Ok(Self { inner: Some(d) }),
            // The wat call site's span (threaded from `eval_io_temp_dir_new`) locates this OS error
            // at the user's `(:wat::io::TempDir/new)`, not at a coarse `rust_caller_span!()`.
            Err(e) => Err(RuntimeError::new(span.clone(), RuntimeErrorKind::MalformedForm {
                head: ":wat::io::TempDir/new".into(),
                reason: format!("create temp dir: {e}")
            })),
        }
    }

    pub fn path(&self) -> Result<String, RuntimeError> {
        match &self.inner {
            Some(d) => Ok(d.path().display().to_string()),
            // arc 138: no span — runtime invariant; no WatAST at this call depth
            None => Err(RuntimeError::new(crate::rust_caller_span!(), RuntimeErrorKind::MalformedForm {
                head: ":wat::io::TempDir/path".into(),
                reason: "TempDir already dropped".into()
            })),
        }
    }
}

// Constructors + accessors via the RustOpaque machinery (same
// shape `wat-telemetry-sqlite`'s cursor uses for its hand-rolled
// thread-owned types). Each wat call lands in one of these eval
// functions; runtime.rs dispatches via keyword head.

pub fn eval_io_temp_file_new(
    _args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let f = WatTempFile::new(list_span)?;
    Ok(crate::rust_deps::make_rust_opaque(
        ":wat::io::TempFile",
        crate::rust_deps::ThreadOwnedCell::new(f),
    ))
}

pub fn eval_io_temp_file_path(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::TempFile/path";
    arity(op, args, 1, list_span)?;
    let v = eval(&args[0], env, sym)?.value_owned();
    let inner = crate::rust_deps::rust_opaque_arc(&v, ":wat::io::TempFile", op, args[0].span().clone())?;
    let cell: &crate::rust_deps::ThreadOwnedCell<WatTempFile> =
        crate::rust_deps::downcast_ref_opaque(&inner, ":wat::io::TempFile", op, args[0].span().clone())?;
    let s = cell.with_ref(op, |f| f.path())??;
    Ok(Value::String(Arc::new(s)))
}

pub fn eval_io_temp_dir_new(
    _args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let d = WatTempDir::new(list_span)?;
    Ok(crate::rust_deps::make_rust_opaque(
        ":wat::io::TempDir",
        crate::rust_deps::ThreadOwnedCell::new(d),
    ))
}

pub fn eval_io_temp_dir_path(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::TempDir/path";
    arity(op, args, 1, list_span)?;
    let v = eval(&args[0], env, sym)?.value_owned();
    let inner = crate::rust_deps::rust_opaque_arc(&v, ":wat::io::TempDir", op, args[0].span().clone())?;
    let cell: &crate::rust_deps::ThreadOwnedCell<WatTempDir> =
        crate::rust_deps::downcast_ref_opaque(&inner, ":wat::io::TempDir", op, args[0].span().clone())?;
    let s = cell.with_ref(op, |d| d.path())??;
    Ok(Value::String(Arc::new(s)))
}

// ─── read-file (arc 093 follow-on, dispatcher pattern) ───────────────────
//
// `(:wat::io::read-file <path-string>) -> :String` — return the
// contents of `<path>` as a String. Routes through the SymbolTable's
// SourceLoader so the same capability discipline that gates
// `:wat::load-file!` / `:wat::eval-file!` applies (FsLoader for
// the wat-cli; ScopedLoader for sandboxed scripts; InMemoryLoader
// in tests). Panics if no loader is attached (the host didn't
// install one — programmer error, not data-flow).
//
// First consumer: dispatcher-style scripts that read EDN from
// stdin specifying both a data path and a query-program path,
// then read the program's source so they can hand it to
// :wat::kernel::run-sandboxed. Useful generally for any wat
// script that wants to operate on file content as a string.
pub fn eval_io_read_file(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::read-file";
    arity(op, args, 1, list_span)?;
    let path = expect_string(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let loader = sym.source_loader().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: op.into(),
        reason: "no SourceLoader attached to SymbolTable; \
                 the host must provide one (FsLoader / ScopedLoader / InMemoryLoader)"
            .into()
    }))?;
    let loaded = loader.fetch_source_file(&path, None).map_err(|e| {
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("loader fetch_source_file({path:?}): {e}")
        })
    })?;
    Ok(Value::String(Arc::new(loaded.source)))
}

/// `(:wat::io::list-dir path)` → `(:wat::core::Vector :- [String])`.
///
/// Enumerate the directory at `path`; return each entry as a FULL path
/// (the OS-level `entry.path()` already joins the input path with the
/// entry name — `read_dir("wat")` yields `"wat/fix.wat"` etc.).
/// Returns a clean RuntimeError (MalformedForm) if the path does not
/// exist or is not a directory.
pub fn eval_io_list_dir(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::io::list-dir";
    arity(op, args, 1, list_span)?;
    let path = expect_string(op, eval(&args[0], env, sym)?, args[0].span().clone())?;
    let read_dir_iter = std::fs::read_dir(path.as_str()).map_err(|e| {
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: format!("read_dir({path:?}): {e}")
        })
    })?;
    let mut entries: Vec<Value> = Vec::new();
    for entry_result in read_dir_iter {
        let entry = entry_result.map_err(|e| {
            RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!("read_dir entry error: {e}")
            })
        })?;
        let full_path = entry.path().to_string_lossy().into_owned();
        entries.push(Value::String(Arc::new(full_path)));
    }
    Ok(Value::Vec(Arc::new(entries)))
}

/// `(:wat::stdlib::sources)` → `(:wat::core::Vector :- [(Vector :- [String])])`
///
/// Returns the baked stdlib load order as a vector of `[path, source]` pairs.
/// Each inner vector has exactly two elements: the path string and the full
/// source string, in `STDLIB_FILES` order. Zero args; pure (no I/O).
/// Consumer: `wat/deporder.wat` wraps these into `SourceFile` records and
/// verifies the load order respects all eval-time dependencies.
pub fn eval_stdlib_sources(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let op = ":wat::stdlib::sources";
    arity(op, args, 0, list_span)?;
    let pairs: Vec<Value> = crate::stdlib::stdlib_files()
        .iter()
        .map(|ws| {
            Value::Vec(Arc::new(vec![
                Value::String(Arc::new(ws.path.to_string())),
                Value::String(Arc::new(ws.source.to_string())),
            ]))
        })
        .collect();
    Ok(Value::Vec(Arc::new(pairs)))
}

// ─── Unit tests for pipe-backed IO (arc 012 slice 1) ─────────────────────

#[cfg(test)]
mod pipe_tests {
    use super::*;
    use std::os::fd::FromRawFd;

    /// Build a fresh `pipe2(O_CLOEXEC)` pair wrapped as our typed ends.
    fn make_pipe() -> (PipeWriter, PipeReader) {
        let mut fds = [0i32; 2];
        let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
        assert_eq!(
            ret,
            0,
            "libc::pipe2 failed: {}",
            std::io::Error::last_os_error()
        );
        let reader_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let writer_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        (
            PipeWriter::from_owned_fd(writer_fd),
            PipeReader::from_owned_fd(reader_fd),
        )
    }

    #[test]
    fn round_trip_bytes() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"hello", s.clone()).expect("write");
        drop(w); // close writer so read_all sees EOF
        let got = r.read_all(s).expect("read_all");
        assert_eq!(got, b"hello");
    }

    #[test]
    fn read_returns_partial() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"abcdef", s.clone()).expect("write");
        // Ask for 3 of 6 available bytes — read(n) returns what's ready.
        let got = r.read(3, s.clone()).expect("read").expect("not EOF");
        assert_eq!(got, b"abc");
        let got = r.read(3, s).expect("read").expect("not EOF");
        assert_eq!(got, b"def");
    }

    #[test]
    fn read_all_eof_when_writer_dropped() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"once", s.clone()).expect("write");
        drop(w);
        let got = r.read_all(s.clone()).expect("read_all");
        assert_eq!(got, b"once");
        // Re-reading after EOF returns empty.
        let again = r.read_all(s).expect("read_all again");
        assert_eq!(again, Vec::<u8>::new());
    }

    #[test]
    fn read_returns_none_on_eof() {
        let (w, r) = make_pipe();
        drop(w);
        let got = r.read(16, crate::rust_caller_span!()).expect("read");
        assert!(got.is_none(), "expected None on EOF; got {:?}", got);
    }

    #[test]
    fn read_line_lf() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"first\nsecond\n", s.clone()).expect("write");
        drop(w);
        assert_eq!(r.read_line(s.clone()).expect("line1"), Some("first".to_string()));
        assert_eq!(r.read_line(s.clone()).expect("line2"), Some("second".to_string()));
        assert_eq!(r.read_line(s).expect("eof"), None);
    }

    #[test]
    fn read_line_crlf_stripped() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"win\r\nline\r\n", s.clone()).expect("write");
        drop(w);
        assert_eq!(r.read_line(s.clone()).expect("line1"), Some("win".to_string()));
        assert_eq!(r.read_line(s.clone()).expect("line2"), Some("line".to_string()));
        assert_eq!(r.read_line(s).expect("eof"), None);
    }

    #[test]
    fn read_line_no_trailing_newline() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        w.write_all(b"bare", s.clone()).expect("write");
        drop(w);
        assert_eq!(r.read_line(s.clone()).expect("bare"), Some("bare".to_string()));
        assert_eq!(r.read_line(s).expect("eof"), None);
    }

    #[test]
    fn rewind_is_error() {
        let (_w, r) = make_pipe();
        let err = r.rewind(crate::rust_caller_span!()).expect_err("pipe rewind must error");
        match err.kind() {
            RuntimeErrorKind::MalformedForm { head, .. } => {
                assert_eq!(head, ":wat::io::rewind");
            }
            _ => panic!("expected MalformedForm; got {:?}", err),
        }
    }

    #[test]
    fn write_returns_count() {
        let (w, r) = make_pipe();
        let s = crate::rust_caller_span!();
        let n = w.write(b"abc", s.clone()).expect("write");
        assert_eq!(n, 3);
        drop(w);
        assert_eq!(r.read_all(s).expect("read_all"), b"abc");
    }

    #[test]
    fn flush_is_ok() {
        let (w, _r) = make_pipe();
        w.flush(crate::rust_caller_span!()).expect("flush");
    }

    #[test]
    fn send_sync_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PipeReader>();
        assert_send_sync::<PipeWriter>();
    }
}

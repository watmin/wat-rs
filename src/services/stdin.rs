//! `:wat::kernel::StdInService` — substrate runtime service that owns
//! fd 0 (or a test-supplied fd), reads line-delimited EDN, parses
//! each line to [`holon::HolonAST`], and dispatches the parsed atom
//! to a registered per-thread consumer.
//!
//! # Pattern (minted here, reused by 1f-ii / 1f-iii)
//!
//! - **Singleton boot:** [`start_stdin_service`] is idempotent;
//!   first call spawns the service thread; second call returns the
//!   same `&'static` handle. Wired into wat-cli boot in slice 1f-iv.
//! - **Test spawn:** [`StdInService::spawn_for_test`] returns a
//!   fresh, non-singleton handle reading from a caller-supplied
//!   `RawFd`. The static fd 0 belongs to the singleton; tests use
//!   their own pipe ends so multiple cargo-test processes don't
//!   contend over the host's real fd 0 and so each test is hermetic.
//! - **Per-thread registration:** consumers call
//!   [`StdInServiceHandle::register`] with their `ThreadId`; service
//!   returns a [`crossbeam_channel::Receiver`] that yields parsed
//!   atoms (`Some(HolonAST)`) and an explicit `None` on EOF.
//!   Calling [`StdInServiceHandle::unregister`] drops the channel.
//!
//! # Internal shape
//!
//! The service worker `poll(2)`s on two fds:
//! - `input_fd` — the data side. POLLIN means bytes are available;
//!   the worker reads up to a buffer, accumulates into a line
//!   buffer, splits on `\n`, hands each completed line to
//!   [`crate::edn_shim::read_holon_ast_natural`], and dispatches.
//!   POLLHUP / read-returning-zero is EOF.
//! - `control_pipe_read_fd` — the wakeup side. POLLIN means the
//!   control crossbeam channel has at least one queued message; the
//!   worker drains the channel and updates its consumer registry.
//!   The pipe carries no payload; its only job is to wake the poll.
//!   This is the "self-pipe trick" — register/unregister calls
//!   write one byte to `control_pipe_write_fd` to break the worker
//!   out of poll without busy-waiting.
//!
//! # Dispatch policy
//!
//! Slice 1f-i ships **single-consumer dispatch**: each parsed atom
//! goes to the FIRST registered consumer (in registration order).
//! With one registered consumer (the typical 1f-i case — main
//! thread or one test consumer), this is unambiguous. Multi-consumer
//! routing (round-robin, topic-based, ownership-leasing) is out of
//! scope for 1f-i; slice 1g + future arcs decide. Documented in
//! SCORE-SLICE-1F-I per the BRIEF.
//!
//! # Zero Mutex
//!
//! Per `docs/ZERO-MUTEX.md` Tier 3 (program-with-mailbox): the
//! service IS the mailbox. Its registry and line-buffer live on the
//! worker thread; mutation happens only inside the loop body; no
//! lock primitive is needed because no other thread reaches in. The
//! handle exposes `register`/`unregister`/`shutdown` over a
//! crossbeam channel + self-pipe pair — atomics + `OnceLock` +
//! `Arc` are the only shared-state primitives. Zero `Mutex`,
//! `RwLock`, `CondVar`.

use crate::edn_shim::read_holon_ast_natural;
use crossbeam_channel::{unbounded, Receiver, Sender};
use holon::HolonAST;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::{self, ThreadId};

// ─── Public API ─────────────────────────────────────────────────────────────

/// Singleton handle to the process-wide [`StdInService`] reading
/// from fd 0. Idempotent; the first call spawns the service thread,
/// subsequent calls return the same `&'static` handle.
///
/// Wat-cli wires this into boot in slice 1f-iv. In-process tests
/// that exercise the singleton path inherit one shared service for
/// the cargo-test process; tests that need isolation should reach
/// for [`StdInService::spawn_for_test`] instead.
pub fn start_stdin_service() -> &'static StdInServiceHandle {
    static SINGLETON: OnceLock<StdInServiceHandle> = OnceLock::new();
    SINGLETON.get_or_init(|| StdInService::spawn_internal(libc::STDIN_FILENO))
}

/// Per-thread service-control message. Sent over the handle's
/// crossbeam control channel; the worker drains the channel after
/// being woken by a self-pipe write.
#[derive(Debug)]
pub enum ControlMsg {
    /// Register a consumer for `thread_id`. The service stores the
    /// `Sender` end and routes parsed atoms to it.
    Register {
        thread_id: ThreadId,
        sender: Sender<Option<Arc<HolonAST>>>,
    },
    /// Unregister the consumer for `thread_id`. The stored
    /// `Sender` drops; the corresponding `Receiver` will see
    /// disconnect.
    Unregister { thread_id: ThreadId },
    /// Shutdown the service worker thread immediately. Notifies
    /// every registered consumer with `None` and exits the poll
    /// loop. Used by [`StdInService::spawn_for_test`] handles when
    /// they're dropped — the singleton handle is `&'static` and
    /// never receives this.
    Shutdown,
}

/// Service handle. Holds the control-channel sender + the self-pipe
/// write fd. Cheaply cloneable via reference (the singleton is
/// `&'static`); test handles are owned values whose `Drop` issues
/// `Shutdown` and lets the worker terminate.
pub struct StdInServiceHandle {
    /// Crossbeam sender for control messages. The worker holds the
    /// matching receiver and drains it after being woken.
    control_tx: Sender<ControlMsg>,

    /// Write end of the self-pipe. Writing one byte here makes the
    /// worker's `poll(2)` return with POLLIN on the control fd; the
    /// worker then drains `control_tx`'s matching receiver.
    /// `OwnedFd` so `Drop` closes the fd if the handle is dropped.
    control_pipe_write_fd: OwnedFd,

    /// True once `Shutdown` has been signaled. Idempotent — we
    /// accept double-drop attempts on test handles silently.
    shutdown_signaled: AtomicBool,

    /// True for handles produced by [`StdInService::spawn_for_test`].
    /// Drop signals shutdown; singleton drop is a no-op (the
    /// `&'static` handle is never dropped in practice — the
    /// `OnceLock`-stored value lives the process lifetime).
    is_test_handle: bool,
}

impl StdInServiceHandle {
    /// Register a consumer thread. Returns a [`Receiver`] that
    /// yields:
    /// - `Some(Arc<HolonAST>)` for each parsed line of input
    /// - `None` once when the input fd EOFs (the service notifies
    ///   every registered consumer on EOF before exiting its loop)
    /// - channel-disconnect after the service exits (the worker's
    ///   stored `Sender` drops)
    ///
    /// The control message is queued and the worker is woken via
    /// the self-pipe; registration is therefore async — if the
    /// caller writes to the input fd immediately after `register`
    /// returns, the worker may still be processing the registration
    /// when the data arrives. For deterministic test setup,
    /// register before producing data and rely on the channel's FIFO
    /// to deliver in order.
    pub fn register(&self, thread_id: ThreadId) -> Receiver<Option<Arc<HolonAST>>> {
        let (tx, rx) = unbounded();
        // Crossbeam send only fails if the worker has dropped the
        // receiver (worker exited). We swallow that — the caller's
        // returned Receiver will simply never fire. A failed wakeup
        // on a dead worker is moot.
        let _ = self.control_tx.send(ControlMsg::Register {
            thread_id,
            sender: tx,
        });
        self.wake_worker();
        rx
    }

    /// Unregister the consumer for `thread_id`. Drops the worker's
    /// stored sender; the corresponding receiver sees disconnect.
    /// Idempotent — unregistering an unknown thread_id is a no-op.
    pub fn unregister(&self, thread_id: ThreadId) {
        let _ = self
            .control_tx
            .send(ControlMsg::Unregister { thread_id });
        self.wake_worker();
    }

    /// Internal: wake the worker by writing one byte to the
    /// self-pipe. Errors are swallowed — pipe-write fails only if
    /// the worker has closed the read end (i.e., it has already
    /// shut down). In that case there's nothing to wake.
    fn wake_worker(&self) {
        let buf: [u8; 1] = [0];
        // SAFETY: control_pipe_write_fd is a valid open fd we own;
        // libc::write with a 1-byte buffer is safe.
        unsafe {
            let _ = libc::write(
                self.control_pipe_write_fd.as_raw_fd(),
                buf.as_ptr() as *const libc::c_void,
                1,
            );
        }
    }
}

impl Drop for StdInServiceHandle {
    fn drop(&mut self) {
        if !self.is_test_handle {
            // Singleton — never actually dropped (OnceLock holds
            // it for process lifetime); but be defensive.
            return;
        }
        if self.shutdown_signaled.swap(true, Ordering::SeqCst) {
            return;
        }
        let _ = self.control_tx.send(ControlMsg::Shutdown);
        self.wake_worker();
    }
}

// ─── Service struct ─────────────────────────────────────────────────────────

/// Marker type for the service. Callers don't construct this
/// directly — they reach for [`start_stdin_service`] (production)
/// or [`StdInService::spawn_for_test`] (Rust integration tests).
pub struct StdInService;

impl StdInService {
    /// Spawn a fresh, non-singleton StdInService reading from
    /// `input_fd`. Returns a [`StdInServiceHandle`] whose `Drop`
    /// shuts down the worker.
    ///
    /// Designed for hermetic Rust integration tests: each test
    /// allocates its own pipe pair, hands the read end to
    /// `spawn_for_test`, writes to the write end to feed input, and
    /// drops the handle to clean up. This shape lets multiple tests
    /// run in the same cargo-test process without contending over
    /// the singleton's fd 0 or each other.
    ///
    /// `input_fd` ownership: the caller retains ownership. The
    /// service uses the fd via `libc::read`/`libc::poll` but does
    /// not close it — the caller's [`OwnedFd`] (or equivalent)
    /// outlives the service or the test cleans up explicitly.
    pub fn spawn_for_test(input_fd: RawFd) -> StdInServiceHandle {
        let mut handle = Self::spawn_internal(input_fd);
        handle.is_test_handle = true;
        handle
    }

    /// Internal common path: allocate the self-pipe, allocate the
    /// crossbeam control channel, spawn the worker, return the
    /// handle. Panics on `pipe(2)` failure with a diagnostic — the
    /// service can't operate without its self-pipe, and silent
    /// degradation would mask the failure (per
    /// `feedback_shim_panic_vs_option.md` — construction-time
    /// failures panic with diagnostic).
    fn spawn_internal(input_fd: RawFd) -> StdInServiceHandle {
        // Allocate the self-pipe via libc::pipe(2).
        let mut fds = [0i32; 2];
        // SAFETY: libc::pipe(2) writes two fds into the array.
        let ret = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if ret != 0 {
            let err = std::io::Error::last_os_error();
            panic!(
                "StdInService: libc::pipe(2) failed: {} \
                 (this is a process-level resource exhaustion or \
                 kernel-config failure; the service cannot operate \
                 without its control self-pipe)",
                err
            );
        }
        // SAFETY: pipe(2) returned 0; fds[0] (read) and fds[1]
        // (write) are freshly opened fds we now own; OwnedFd takes
        // ownership and Drop will close them.
        let control_pipe_read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let control_pipe_write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };

        let (control_tx, control_rx) = unbounded::<ControlMsg>();

        // Spawn the worker. The worker takes ownership of the
        // control_pipe_read_fd OwnedFd (its Drop closes the fd when
        // the worker exits). The handle keeps control_pipe_write_fd
        // (its Drop closes the fd when the handle is dropped — the
        // worker observes POLLHUP on the read side and exits if it
        // hadn't already on Shutdown).
        thread::Builder::new()
            .name("wat::kernel::StdInService".into())
            .spawn(move || {
                run_service_loop(input_fd, control_pipe_read_fd, control_rx);
            })
            // Failing to spawn a kernel-internal service thread is
            // a process-fatal condition; panic with diagnostic.
            .expect(
                "StdInService: failed to spawn worker thread; \
                 process is out of threads or memory and the \
                 service cannot operate",
            );

        StdInServiceHandle {
            control_tx,
            control_pipe_write_fd,
            shutdown_signaled: AtomicBool::new(false),
            is_test_handle: false,
        }
    }
}

// ─── Worker loop ────────────────────────────────────────────────────────────

/// Per-consumer registry entry. Order matters for slice 1f-i's
/// single-consumer dispatch policy (first-registered wins).
struct Consumer {
    thread_id: ThreadId,
    sender: Sender<Option<Arc<HolonAST>>>,
}

/// Worker entry point. Owns:
/// - `input_fd`: the data fd (fd 0 or a test pipe)
/// - `control_pipe_read_fd`: the self-pipe read end (drains on wake)
/// - `control_rx`: the crossbeam control channel
///
/// The worker runs `libc::poll(2)` on `[input_fd, control_pipe_read_fd]`
/// and dispatches based on which fd fires.
fn run_service_loop(
    input_fd: RawFd,
    control_pipe_read_fd: OwnedFd,
    control_rx: Receiver<ControlMsg>,
) {
    let mut consumers: Vec<Consumer> = Vec::new();
    let mut line_buf: Vec<u8> = Vec::with_capacity(256);

    loop {
        // Build the pollfd array. POLLIN on both; we don't watch
        // POLLOUT (we never write to either fd — control via
        // crossbeam channel, input fd is read-only).
        let mut pollfds = [
            libc::pollfd {
                fd: input_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: control_pipe_read_fd.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        // -1 = block indefinitely. The self-pipe wakes us on
        // control activity; the input fd wakes us on data; nothing
        // else can wake us. No CPU spin; no lost wakeup window.
        // SAFETY: pollfds is a valid stack array of 2 pollfd
        // structs; libc::poll reads/writes them in place.
        let ret = unsafe { libc::poll(pollfds.as_mut_ptr(), pollfds.len() as libc::nfds_t, -1) };

        if ret < 0 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            // EINTR — interrupted by a signal; just re-poll.
            if errno == libc::EINTR {
                continue;
            }
            // Any other error is process-fatal in concept (the
            // service can't recover), but the BRIEF says don't
            // workaround substrate gaps; surface as a notification
            // to consumers and exit. Sending None to all consumers
            // mimics EOF — they observe end-of-stream and can
            // decide what to do.
            notify_eof_and_drop(&mut consumers);
            return;
        }
        if ret == 0 {
            // Timeout — but we passed -1 (infinite). Should never
            // happen; treat as benign re-poll.
            continue;
        }

        // Order matters subtly: drain control FIRST so any
        // pending register lands before we route freshly-read
        // input. With two POLLIN events queued in one poll wake,
        // the BRIEF's "interleave data + control" test (row H)
        // expects ordered register-then-data flow.
        if pollfds[1].revents & (libc::POLLIN | libc::POLLHUP) != 0 {
            // Drain self-pipe bytes (we don't care about the
            // content; the bytes are wakeup signals).
            drain_self_pipe(control_pipe_read_fd.as_raw_fd());
            // Drain crossbeam control channel.
            while let Ok(msg) = control_rx.try_recv() {
                match msg {
                    ControlMsg::Register { thread_id, sender } => {
                        consumers.push(Consumer { thread_id, sender });
                    }
                    ControlMsg::Unregister { thread_id } => {
                        consumers.retain(|c| c.thread_id != thread_id);
                    }
                    ControlMsg::Shutdown => {
                        notify_eof_and_drop(&mut consumers);
                        return;
                    }
                }
            }
        }

        if pollfds[0].revents & libc::POLLIN != 0 {
            match read_into_line_buf(input_fd, &mut line_buf, &mut consumers) {
                ReadOutcome::Continue => {}
                ReadOutcome::Eof => {
                    notify_eof_and_drop(&mut consumers);
                    return;
                }
            }
        } else if pollfds[0].revents & libc::POLLHUP != 0 {
            // POLLHUP without POLLIN — fd closed with no pending
            // data. Treat as EOF.
            notify_eof_and_drop(&mut consumers);
            return;
        }
    }
}

enum ReadOutcome {
    Continue,
    Eof,
}

/// Read available bytes from `input_fd` into `line_buf`, splitting
/// completed lines off the front and dispatching each parsed atom
/// to the first registered consumer. Returns [`ReadOutcome::Eof`]
/// when read returns 0.
fn read_into_line_buf(
    input_fd: RawFd,
    line_buf: &mut Vec<u8>,
    consumers: &mut Vec<Consumer>,
) -> ReadOutcome {
    let mut buf = [0u8; 4096];
    // SAFETY: buf is a valid mutable byte buffer; libc::read writes
    // up to buf.len() bytes into it.
    let n = unsafe { libc::read(input_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
    if n < 0 {
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        if errno == libc::EINTR || errno == libc::EAGAIN || errno == libc::EWOULDBLOCK {
            // Spurious wake; nothing to read this round.
            return ReadOutcome::Continue;
        }
        // Other read errors: treat as EOF (the fd is bad; we can't
        // recover from inside the worker without the BRIEF's
        // explicit StdErrService cascade, which slice 1f-iii owns).
        return ReadOutcome::Eof;
    }
    if n == 0 {
        // Drain any final (newline-terminated) line in line_buf.
        flush_pending_line(line_buf, consumers);
        return ReadOutcome::Eof;
    }
    line_buf.extend_from_slice(&buf[..n as usize]);

    // Split on newline; each completed line is one EDN value.
    while let Some(pos) = line_buf.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = line_buf.drain(..=pos).collect();
        // Strip trailing `\n` (and `\r` if present for CRLF).
        let line_str_end = if pos > 0 && line[pos - 1] == b'\r' {
            pos - 1
        } else {
            pos
        };
        let line_str = match std::str::from_utf8(&line[..line_str_end]) {
            Ok(s) => s,
            Err(_) => {
                // Malformed UTF-8 — slice 1f-iii cascade owns the
                // diagnostic emit path. For 1f-i, the BRIEF allows
                // panic-with-diagnostic. We choose drop-the-line
                // instead: panic would tear down the service mid-
                // flight and break tests that exercise normal
                // input. Document the choice in SCORE.
                continue;
            }
        };
        // Empty line (CR-LF or back-to-back `\n`) — skip.
        if line_str.is_empty() {
            continue;
        }
        let atom = match read_holon_ast_natural(line_str) {
            Ok(a) => a,
            Err(_) => {
                // Same rationale as malformed-UTF-8: drop the line
                // for 1f-i. Slice 1f-iii integration cascades the
                // parse error to StdErrService.
                continue;
            }
        };
        dispatch_to_first(consumers, atom);
    }
    ReadOutcome::Continue
}

/// Flush any line-terminated bytes left in `line_buf` at EOF.
/// Bytes WITHOUT a trailing newline are dropped — the protocol is
/// line-delimited; partial trailing lines are not valid messages.
/// Documented in SCORE per honest delta.
fn flush_pending_line(line_buf: &mut Vec<u8>, _consumers: &mut [Consumer]) {
    line_buf.clear();
}

/// Send `Some(atom)` to the first registered consumer (registration
/// order). Drops dead consumers (whose receivers have been dropped)
/// silently — the next iteration's registry will not include them.
fn dispatch_to_first(consumers: &mut Vec<Consumer>, atom: Arc<HolonAST>) {
    while let Some(c) = consumers.first() {
        match c.sender.send(Some(Arc::clone(&atom))) {
            Ok(()) => return,
            Err(_) => {
                // Receiver dropped — remove and try the next.
                consumers.remove(0);
            }
        }
    }
    // No consumers registered — drop the atom silently. The BRIEF's
    // dispatch policy for slice 1f-i assumes at least one consumer
    // is registered before data arrives; documented in SCORE.
}

/// On EOF / shutdown: send `None` to every registered consumer and
/// drop the registry. Each consumer's receiver observes one final
/// `None` followed by channel-disconnect when our Sender drops.
fn notify_eof_and_drop(consumers: &mut Vec<Consumer>) {
    for c in consumers.iter() {
        let _ = c.sender.send(None);
    }
    consumers.clear();
}

/// Drain the self-pipe's read fd. We don't care about the bytes;
/// the byte's only purpose was to wake `poll(2)`. Read until
/// would-block / EOF / EINTR.
fn drain_self_pipe(read_fd: RawFd) {
    let mut buf = [0u8; 64];
    loop {
        // SAFETY: buf is a valid mutable byte buffer.
        let n = unsafe { libc::read(read_fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n <= 0 {
            // 0 = EOF (write end closed; service is shutting down)
            // <0 = EINTR / EAGAIN / fatal — any of these means
            // "stop draining this round." The next poll wake will
            // re-enter the drain.
            return;
        }
        if (n as usize) < buf.len() {
            return;
        }
        // Full buffer; loop to drain more.
    }
}

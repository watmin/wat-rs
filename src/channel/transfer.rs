//! The movement surface: typed send/recv ops, outcomes, and channel-pair
//! constructors. Lifted verbatim from `src/typed_channel.rs` at Stone 6.1;
//! behavior identical.

use crate::channel::inner::{ReceiverInner, SenderInner};
use crate::span::Span;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Outcome of a typed-channel send. Mirrors crossbeam's
/// `Send::send` shape but carries enough info for the wat-level
/// Result wrapper to distinguish disconnect vs other failure modes.
#[derive(Debug)]
pub enum SendOutcome {
    /// The value landed (crossbeam: queued; pipe: bytes flushed
    /// at the kernel boundary).
    Ok,
    /// The peer is gone (crossbeam: every receiver dropped; pipe:
    /// reader closed → EPIPE).
    Disconnected,
}

/// Outcome of a typed-channel recv. Mirrors crossbeam's
/// `Receiver::recv` shape — `Some(Value)` on a value, `None` on
/// clean disconnect (every sender dropped / writer closed +
/// no buffered data).
#[derive(Debug)]
pub enum RecvOutcome {
    /// A value flowed.
    Value(crate::runtime::Value),
    /// Clean shutdown — every sender has dropped (crossbeam) or
    /// the pipe writer closed and no further bytes are in flight
    /// (pipe-fd).
    Disconnected,
    /// The pipe carried bytes that didn't parse as EDN. Tier 2
    /// only — crossbeam can't surface this. Carries the parse
    /// diagnostic for the caller to surface as a wat-level error.
    DecodeError(String),
    /// arc 170 Slice A: process-wide shutdown signal fired.
    /// Distinguishable from Disconnected: the channel didn't lose its
    /// partner — the process is shutting down. Slice B wires recv to
    /// surface this; Slice A only adds the variant.
    Shutdown,
}

/// Send a typed `Value` through a transport-polymorphic Sender.
///
/// - Tier 1 (Crossbeam): zero-copy enqueue.
/// - Tier 2 (PipeFd): EDN-encode + append `'\n'` + write to fd.
///   The `types` registry is consulted by `value_to_edn_with` so
///   tagged structs / enums round-trip with their type names.
///
/// Span is used for error reporting on pipe-write failures (so
/// the caller's source span surfaces in the diagnostic).
pub fn typed_send(
    sender: &SenderInner,
    value: crate::runtime::Value,
    types: Option<&crate::types::TypeEnv>,
    span: Span,
) -> SendOutcome {
    match sender {
        SenderInner::Comms { sender: tx, closed } => {
            // Arc 170 slice 3 Gap B — check closed flag before
            // attempting transport send. Acquire ordering pairs with
            // the SeqCst store in sender_close so this thread sees
            // the flag update from any concurrent close call.
            if closed.load(Ordering::Acquire) {
                return SendOutcome::Disconnected;
            }
            // comms::thread::Sender::send returns Err(SendError(value)) when
            // all receivers are dropped — maps 1:1 to the old crossbeam
            // SendError (Arc 214 Stone 5.1, STOP-1 check: same outcome surface).
            match tx.send(value) {
                Ok(()) => SendOutcome::Ok,
                Err(_) => SendOutcome::Disconnected,
            }
        }
        SenderInner::PipeFd { writer, closed } => {
            // Arc 170 slice 3 Gap B — check closed flag before write.
            if closed.load(Ordering::Acquire) {
                return SendOutcome::Disconnected;
            }
            let edn = crate::edn_shim::value_to_edn_with(&value, types);
            let mut payload = wat_edn::write(&edn);
            payload.push('\n');
            match writer.write_all(payload.as_bytes(), span) {
                Ok(()) => SendOutcome::Ok,
                // Pipe writes fail with EPIPE when the reader is
                // closed; surface uniformly as Disconnected so the
                // wat-level Result.Err shape is consistent across
                // transports.
                Err(_) => SendOutcome::Disconnected,
            }
        }
    }
}

/// Arc 170 slice 3 Gap B — signal end-of-stream on the send side
/// without dropping the Sender Value.
///
/// Sets the `closed` flag to `true` (idempotent). For Crossbeam
/// senders, the flag is sufficient — subsequent `typed_send` calls
/// check it and return `SendOutcome::Disconnected`. For PipeFd
/// senders, also calls `writer.close()` which releases the
/// underlying fd via `libc::close(2)` so the peer reader sees EOF
/// on its next read (the same `PipeWriter::close` that
/// `IOWriter/close` calls, per `src/io.rs:665`).
///
/// Calling `sender_close` twice is safe (idempotent): the second
/// call finds the flag already set; for PipeFd the `PipeWriter::close`
/// impl atomically swaps fd to -1 and no-ops if already -1.
///
/// Returns `Ok(())` always — callers convert to `Value::Unit` (nil).
pub fn sender_close(
    sender: &SenderInner,
    span: Span,
) -> Result<(), crate::value::RuntimeError> {
    match sender {
        SenderInner::Comms { closed, .. } => {
            // SeqCst store ensures all threads see the flag; Acquire
            // load in typed_send pairs with this.
            // Arc 214 Stone 5.1 — the comms::thread::Sender::close takes
            // ownership (move semantics), so we only set the flag here.
            // The actual channel disconnect happens when the Arc<SenderInner>
            // drops (i.e., when the last Sender Value is gone). The closed
            // flag gates all future typed_send calls immediately.
            closed.store(true, Ordering::SeqCst);
            Ok(())
        }
        SenderInner::PipeFd { writer, closed } => {
            // Set the flag first so typed_send stops immediately.
            closed.store(true, Ordering::SeqCst);
            // Release the fd — the peer reader's next read sees EOF.
            // PipeWriter::close is idempotent (atomically swaps fd
            // to -1; no-op if already -1). Errors from close(2) are
            // advisory; PipeWriter::close discards them — same policy
            // as IOWriter/close.
            writer.close(span)
        }
    }
}

/// Receive a typed `Value` from a transport-polymorphic Receiver.
///
/// Blocks until a value flows or the peer disconnects.
///
/// - Tier 1 (Crossbeam): blocks on the crossbeam recv, multiplexed
///   against `SHUTDOWN_RX` so a process-wide shutdown signal wakes
///   blocked recvs (arc 170 Slice B). If `SHUTDOWN_RX` is not yet
///   initialized (bootstrap pre-init or test bypass), falls back to
///   bare recv — should not happen in production paths.
/// - Tier 2 (PipeFd): reads one line from the fd, parses as EDN
///   via `read_edn`. The `types` registry interprets `#ns/Name`
///   tags as tagged structs / enums.
pub fn typed_recv(
    receiver: &ReceiverInner,
    types: Option<&crate::types::TypeEnv>,
    span: Span,
) -> RecvOutcome {
    match receiver {
        ReceiverInner::Comms(rx) => {
            // Arc 214 Stone 5.1 — delegate to comms::thread::Receiver::recv()
            // which is already cascade-aware (wires SHUTDOWN_RX internally,
            // with the same bootstrap fallback as the old Crossbeam arm).
            //
            // comms::thread::Receiver::recv() returns Err(RecvError) for both
            // channel disconnect AND substrate shutdown (the comms select merges
            // them). To preserve the RecvOutcome surface exactly we perform a
            // non-blocking check of SHUTDOWN_RX after an Err to distinguish:
            //   - Err(TryRecvError::Disconnected) → SHUTDOWN_TX dropped = shutdown fired
            //   - Err(TryRecvError::Empty)        → data channel disconnected, not shutdown
            match rx.recv() {
                Ok(v) => RecvOutcome::Value(v),
                Err(_) => {
                    let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
                    match shutdown_rx {
                        Some(srx) => match srx.try_recv() {
                            // Shutdown channel drained by comms recv or already
                            // signaled — substrate shutdown is the cause.
                            Ok(_) | Err(crossbeam_channel::TryRecvError::Disconnected) => {
                                RecvOutcome::Shutdown
                            }
                            // Channel empty — data peer disconnected, not shutdown.
                            Err(crossbeam_channel::TryRecvError::Empty) => {
                                RecvOutcome::Disconnected
                            }
                        },
                        // Bootstrap pre-init: no shutdown signal → data disconnect.
                        None => RecvOutcome::Disconnected,
                    }
                }
            }
        }
        ReceiverInner::PipeFd(reader) => {
            // Phase 2 — multiplex on shutdown via OS-level poll.
            // If reader exposes a pollable FD AND the substrate's shutdown
            // broadcast is initialized, poll both; otherwise fall back to
            // bare read_line (non-FD-backed reader, or pre-init bootstrap).
            let pipe_fd_opt = reader.as_raw_fd_for_poll();
            let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
                std::sync::atomic::Ordering::SeqCst,
            );
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
                            events: libc::POLLHUP,
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
                        break;
                    }
                    if n == 0 {
                        // timeout=-1 should never produce n=0; defensively retry.
                        continue;
                    }
                    // Shutdown wins ties per Slice B discipline — process is
                    // going down; honest reporting.
                    if fds[1].revents != 0 {
                        return RecvOutcome::Shutdown;
                    }
                    if fds[0].revents != 0 {
                        break;
                    }
                }
            }
            // Pipe is ready (or no multiplex possible). Read.
            match reader.read_line(span) {
                Ok(Some(line)) => {
                    let trimmed = line.trim_end_matches('\n');
                    match crate::edn_shim::read_edn(trimmed, types) {
                        Ok(v) => RecvOutcome::Value(v),
                        Err(e) => RecvOutcome::DecodeError(format!("{}", e)),
                    }
                }
                Ok(None) => RecvOutcome::Disconnected,
                // A read error (kernel-level, not EOF) is also a
                // disconnect from the wat-level POV — there's nothing
                // useful for the caller to do beyond bail. Caller can
                // distinguish if it cares by inspecting the IOReader
                // directly.
                Err(_) => RecvOutcome::Disconnected,
            }
        }
    }
}

/// Non-blocking variant of [`typed_recv`].
///
/// - Tier 1 (Crossbeam): checks `SHUTDOWN_RX` first (fast path on
///   shutdown active), then `try_recv` on the data channel (arc 170
///   Slice B). On shutdown active → `RecvOutcome::Shutdown`. On data
///   ready → `RecvOutcome::Value`. On empty-or-disconnected →
///   `RecvOutcome::Disconnected`. The order matters: shutdown checked
///   first so it overrides any pending Value (the process is going
///   down; honest reporting).
/// - Tier 2 (PipeFd): Arc 170 Phase 2 — non-blocking poll(timeout=0)
///   on (broadcast_fd, pipe_fd). Shutdown wins ties. On shutdown →
///   `RecvOutcome::Shutdown`. On data ready → falls through to
///   read_line → `RecvOutcome::Value`. On empty (no data, no shutdown)
///   or poll error → `RecvOutcome::Disconnected`. Broadcast fd is
///   checked first so shutdown overrides any pending Value (process
///   is going down; honest reporting).
pub fn typed_try_recv(
    receiver: &ReceiverInner,
    _types: Option<&crate::types::TypeEnv>,
    _span: Span,
) -> RecvOutcome {
    match receiver {
        ReceiverInner::Comms(rx) => {
            // Arc 214 Stone 5.1 — delegate to comms::thread::Receiver::try_recv()
            // which returns Option<T> (None for both Empty and Disconnected, per
            // arc 253 2-state collapse). Preserve the shutdown-first fast path
            // exactly as the old Crossbeam arm did.
            let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
            if let Some(srx) = shutdown_rx {
                // Non-blocking: check shutdown first (fast path on shutdown active).
                // Treat Disconnected on SHUTDOWN_RX the same as a shutdown signal —
                // the sender was dropped, which means trigger_shutdown() ran.
                match srx.try_recv() {
                    Ok(_) | Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        return RecvOutcome::Shutdown;
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => {}
                }
            }
            // comms::thread::try_recv returns Option<T>:
            //   Some(v) → Value
            //   None    → Empty or Disconnected (arc 253 collapse) → Disconnected
            match rx.try_recv() {
                Some(v) => RecvOutcome::Value(v),
                None => RecvOutcome::Disconnected,
            }
        }
        ReceiverInner::PipeFd(reader) => {
            let pipe_fd_opt = reader.as_raw_fd_for_poll();
            let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
                std::sync::atomic::Ordering::SeqCst,
            );
            if let (Some(pfd), true) = (pipe_fd_opt, broadcast_fd >= 0) {
                let mut fds = [
                    libc::pollfd {
                        fd: broadcast_fd,
                        events: libc::POLLHUP,
                        revents: 0,
                    },
                    libc::pollfd {
                        fd: pfd,
                        events: libc::POLLIN | libc::POLLHUP,
                        revents: 0,
                    },
                ];
                let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, 0) };
                if n > 0 {
                    // Shutdown wins ties.
                    if fds[0].revents != 0 {
                        return RecvOutcome::Shutdown;
                    }
                    // Pipe ready — fall through to read_line.
                } else {
                    // n == 0: timeout (no data, no shutdown). Empty.
                    // n < 0: error — Disconnected as the substrate's
                    //   honest "I have no data" signal.
                    return RecvOutcome::Disconnected;
                }
            } else {
                // No multiplex possible — preserve old behavior.
                return RecvOutcome::Disconnected;
            }
            // Pipe ready — try one read.
            match reader.read_line(_span) {
                Ok(Some(line)) => {
                    let trimmed = line.trim_end_matches('\n');
                    match crate::edn_shim::read_edn(trimmed, _types) {
                        Ok(v) => RecvOutcome::Value(v),
                        Err(e) => RecvOutcome::DecodeError(format!("{}", e)),
                    }
                }
                Ok(None) => RecvOutcome::Disconnected,
                Err(_) => RecvOutcome::Disconnected,
            }
        }
    }
}

/// Helper for `:wat::kernel::select` — extracts the underlying
/// `comms::thread::Receiver` if the inner is `Comms`. Returns `None`
/// for `PipeFd` (select is tier-1-only today; piped channels
/// would need an epoll/poll integration that's substrate work
/// for a follow-up arc).
///
/// Arc 214 Stone 5.1 — replaces `try_as_crossbeam_receiver`; the
/// `eval_kernel_select` memory path now registers via
/// `comms::thread::Select` instead of `crossbeam_channel::Select`.
pub fn try_as_comms_receiver(
    receiver: &ReceiverInner,
) -> Option<&crate::comms::thread::Receiver<crate::runtime::Value>> {
    match receiver {
        ReceiverInner::Comms(rx) => Some(rx),
        ReceiverInner::PipeFd(_) => None,
    }
}

/// Allocate a tier-2 (pipe-fd-backed) typed-channel pair for
/// substrate-internal use.
///
/// Creates an OS pipe via `pipe(2)`; wraps the write end as a
/// PipeFd-backed `Sender<T>` Value and the read end as a PipeFd-
/// backed `Receiver<T>` Value. Bytes flowing through the pipe are
/// EDN-encoded by the substrate; user code sees typed Values.
///
/// Returns the pair as a `(Sender<T>, Receiver<T>)` tuple Value
/// — same shape `:wat::kernel::make-channel` returns for the
/// tier-1 case. `T` is phantom at the runtime layer; the type
/// checker enforces homogeneity per FOUNDATION.
///
/// `op` is the caller's wat-level op name for diagnostic
/// attribution (matches the `make-pipe` convention used by
/// fork.rs / spawn.rs).
///
/// Slice 1c surface — Rust-internal helper. The wat-level verb
/// that wires this to a wat-callable (e.g., `make-pipe-channel`)
/// is slice-2 territory if a real consumer demands it; today's
/// users come through `spawn-process` (slice 2) which constructs
/// the Process<I,O> typed-channel handles internally.
pub fn make_pipe_channel_pair(
    op: &'static str,
) -> Result<(crate::runtime::Value, crate::runtime::Value), crate::value::RuntimeError> {
    use crate::channel::inner::{sender_from_pipe, receiver_from_pipe};
    let (read_fd, write_fd) = crate::process::make_pipe(op)?;
    let writer: Arc<dyn crate::io::WatWriter> =
        Arc::new(crate::io::PipeWriter::from_owned_fd(write_fd));
    let reader: Arc<dyn crate::io::WatReader> =
        Arc::new(crate::io::PipeReader::from_owned_fd(read_fd));
    Ok((sender_from_pipe(writer), receiver_from_pipe(reader)))
}

/// Arc 170 Stone C1 — substrate-internal test fixture. Constructs two
/// cross-wired `:wat::kernel::ThreadPeer` struct Values backed by two
/// crossbeam channel pairs.
///
/// The wiring (returned as `(peer_a, peer_b)`):
///   pipe_AB: A writes → B reads     (carries direction A→B)
///   pipe_BA: B writes → A reads     (carries direction B→A)
///
///   peer_a.rx = pipe_BA.receiver   (A pulls what B wrote)
///   peer_a.tx = pipe_AB.sender     (A pushes toward B)
///   peer_b.rx = pipe_AB.receiver   (B pulls what A wrote)
///   peer_b.tx = pipe_BA.sender     (B pushes toward A)
///
/// For a logical `(X, Y)` exchange where A writes X and B writes Y,
/// peer A's type parameters are `<Y, X>` (reads Y, writes X) and peer
/// B's are `<X, Y>` (reads X, writes Y). The substrate construction
/// is type-erased at this layer — the type parameters live in the
/// checker's `TypeEnv` only; the runtime ferries `Value`s.
///
/// This helper is intentionally NOT exposed to wat user code — Stone D's
/// `run-threads` bracket macro is the user-facing path that builds
/// peer pairs (with the type-parameter mirror baked into the macro
/// expansion). Stone C1 only needs in-Rust peer construction for the
/// substrate-layer tests; the `_for_test` suffix preserves that
/// boundary on every grep.
pub fn make_thread_peer_pair_for_test()
    -> (crate::runtime::Value, crate::runtime::Value)
{
    use crate::channel::inner::{receiver_from_comms, sender_from_comms};
    // Arc 214 Stone 5.1 — use comms::thread::pair() (depth-1) instead of
    // bare crossbeam::unbounded. Depth-1 is sufficient for test fixtures.
    let (tx_ab, rx_ab) = crate::comms::thread::pair::<crate::runtime::Value>();
    let (tx_ba, rx_ba) = crate::comms::thread::pair::<crate::runtime::Value>();
    let peer_a = crate::runtime::Value::Struct(Arc::new(crate::runtime::StructValue {
        type_name: ":wat::kernel::ThreadPeer".into(),
        fields: vec![
            receiver_from_comms(rx_ba),
            sender_from_comms(tx_ab),
        ],
    }));
    let peer_b = crate::runtime::Value::Struct(Arc::new(crate::runtime::StructValue {
        type_name: ":wat::kernel::ThreadPeer".into(),
        fields: vec![
            receiver_from_comms(rx_ab),
            sender_from_comms(tx_ba),
        ],
    }));
    (peer_a, peer_b)
}

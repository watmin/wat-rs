//! The movement surface: typed send/recv ops, outcomes, and channel-pair
//! constructors. Lifted verbatim from `src/typed_channel.rs` at Stone 6.1;
//! behavior identical.

use crate::channel::inner::{ReceiverInner, SenderInner};
use crate::span::Span;
use std::sync::atomic::Ordering;

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
///
/// `types` and `span` are unused now that `SenderInner` has a single
/// (Comms) variant — the pipe-fd send arm that consumed them was
/// annihilated (arc 278, dead-send-half cut). Kept in the signature
/// unchanged: `typed_send` still has a live caller at
/// `kernel/address.rs` that passes both.
pub fn typed_send(
    sender: &SenderInner,
    value: crate::runtime::Value,
    _types: Option<&crate::types::TypeEnv>,
    _span: Span,
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
    }
}

/// Arc 170 slice 3 Gap B — signal end-of-stream on the send side
/// without dropping the Sender Value.
///
/// Sets the `closed` flag to `true` (idempotent). For Comms
/// senders, the flag is sufficient — subsequent `typed_send` calls
/// check it and return `SendOutcome::Disconnected`.
///
/// Calling `sender_close` twice is safe (idempotent): the second
/// call finds the flag already set.
///
/// `span` is unused now that `SenderInner` has a single (Comms)
/// variant — the pipe-fd close arm that consumed it was annihilated
/// (arc 278, dead-send-half cut). Kept in the signature unchanged.
///
/// Returns `Ok(())` always — callers convert to `Value::Unit` (nil).
pub fn sender_close(
    sender: &SenderInner,
    _span: Span,
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
///
/// `ctx` (arc 294.g) — the ambient `EncodingCtx` `read_edn` needs to derive a decoded
/// HolonRecord's hologram (the wire no longer carries it). Passed straight through from the
/// caller's `SymbolTable`; `None` only where no live program's ctx is reachable — a decode
/// that then meets a HolonRecord class errors loudly rather than fabricate a wrong-dim one.
pub fn typed_recv(
    receiver: &ReceiverInner,
    types: Option<&crate::types::TypeEnv>,
    span: Span,
    ctx: Option<&crate::value::EncodingCtx>,
) -> RecvOutcome {
    match receiver {
        ReceiverInner::Comms(rx) => {
            // Stone 214 1b-ii-ε — RecvError carries the cause the comms select
            // already computed at the arm level; no SHUTDOWN_RX peek needed.
            match rx.recv() {
                Ok(v) => RecvOutcome::Value(v),
                Err(crate::comms::RecvError::Shutdown) => RecvOutcome::Shutdown,
                Err(crate::comms::RecvError::Disconnected) => RecvOutcome::Disconnected,
                // FrameTooLarge: the peer sent a frame exceeding the cap.
                // The channel is unusable for this peer; treat as a disconnect
                // (the peer should be torn down by the caller before this path).
                // Stays folded into Disconnected here (unchanged) — per the
                // arc 278 contract, FrameTooLarge is NEVER read off the err
                // channel and NEVER folded into Failed; this is that same
                // documented distinct case, not a hidden failure.
                Err(crate::comms::RecvError::FrameTooLarge) => RecvOutcome::Disconnected,
                // Arc 278 no-hidden-failures — transport-tier twin: a raw wire
                // failure (io error / invalid UTF-8 / decode failure / malformed
                // frame) carries its reason through RecvOutcome::DecodeError
                // (the one reason-carrying shape this enum already has) instead
                // of collapsing into a mute Disconnected.
                Err(crate::comms::RecvError::Failed(reason)) => RecvOutcome::DecodeError(reason),
                // Arc 278 RST stone: `PeerCrashed` and its severed twin are
                // `Peer'`-messaging-only signals (`kernel::peer`'s
                // `notify_peer_crashed_best_effort` /
                // `notify_peer_severed_best_effort` send the reserved sentinels;
                // nothing else ever does) — a bare
                // `:wat::kernel::Sender<T>`/`Receiver<T>` channel (this path)
                // never legitimately produces either. Per arc 278 no-hidden-
                // failures, don't silently fold an impossible-in-practice case
                // into a clean `Disconnected`; surface it loudly instead. Both
                // share this arm because the reasoning is one reasoning — not a
                // tidy uniformity: each `Display` still names which sentinel
                // arrived, so the loud report stays specific.
                Err(
                    e @ (crate::comms::RecvError::PeerCrashed
                    | crate::comms::RecvError::PeerSevered),
                ) => RecvOutcome::DecodeError(e.to_string()),
            }
        }
        ReceiverInner::PipeFd(reader) => {
            // Phase 2 — multiplex on shutdown via OS-level poll.
            // Value-framing upgrade: accumulate physical lines until the
            // buffer forms a complete EDN value (edn_frame_status Complete).
            // The poll/shutdown multiplex fires around EACH read_line so
            // shutdown responsiveness is preserved between lines of a
            // multi-line frame (Slice B discipline — shutdown wins ties).
            let pipe_fd_opt = reader.as_raw_fd_for_poll();
            let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(
                std::sync::atomic::Ordering::SeqCst,
            );

            // Inner helper: poll-then-read one physical line.
            // Returns Ok(Some(line)) | Ok(None) for EOF | Err for shutdown.
            // Using a named enum avoids closure escape problems with
            // RecvOutcome::Shutdown.
            enum LineResult {
                Line(String),
                Eof,
                Shutdown,
                Disconnected,
            }
            let read_one_line = || -> LineResult {
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
                        // Shutdown wins ties per Slice B discipline.
                        if fds[1].revents != 0 {
                            return LineResult::Shutdown;
                        }
                        if fds[0].revents != 0 {
                            break;
                        }
                    }
                }
                // Pipe is ready (or no multiplex). Read one physical line.
                match reader.read_line(span.clone()) {
                    Ok(Some(line)) => LineResult::Line(line),
                    Ok(None) => LineResult::Eof,
                    Err(_) => LineResult::Disconnected,
                }
            };

            // Accumulate lines until the buffer is a complete EDN value.
            let mut buf = String::new();
            loop {
                match read_one_line() {
                    LineResult::Shutdown => return RecvOutcome::Shutdown,
                    LineResult::Disconnected => return RecvOutcome::Disconnected,
                    LineResult::Eof => {
                        if buf.is_empty() {
                            return RecvOutcome::Disconnected;
                        }
                        // Truncated frame — writer died mid-value.
                        return RecvOutcome::DecodeError(format!(
                            "EOF mid-frame (truncated EDN value): {:?}",
                            buf
                        ));
                    }
                    LineResult::Line(line) => {
                        // read_line strips the trailing '\n'; re-add it so
                        // the accumulated buffer is valid multi-line EDN.
                        buf.push_str(&line);
                        buf.push('\n');
                        // Bounded-buffer safety cap: reject frames that grow
                        // without bound (broken/malicious peer). Check BEFORE
                        // edn_frame_status — cheaper, and catches even buffers
                        // that would parse. Uses the same constant as
                        // read_framed_edn for consistent behaviour.
                        if buf.len() > crate::edn::render::DEFAULT_MAX_FRAME_BYTES {
                            return RecvOutcome::DecodeError(format!(
                                "EDN frame exceeded {} bytes without completing — message too large or never terminated",
                                buf.len()
                            ));
                        }
                        use crate::edn::render::EdnFrameStatus;
                        match crate::edn::render::edn_frame_status(&buf) {
                            EdnFrameStatus::Incomplete => continue,
                            EdnFrameStatus::Complete => {
                                // Trim trailing newline for read_edn (which
                                // also handles multi-line strings just fine).
                                let trimmed = buf.trim_end_matches('\n');
                                return match crate::edn::render::read_edn(trimmed, types, ctx) {
                                    Ok(v) => RecvOutcome::Value(v),
                                    Err(e) => RecvOutcome::DecodeError(format!("{}", e)),
                                };
                            }
                            EdnFrameStatus::Malformed(msg) => {
                                return RecvOutcome::DecodeError(format!(
                                    "malformed EDN frame: {}",
                                    msg
                                ));
                            }
                        }
                    }
                }
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

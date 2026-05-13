//! Arc 170 slice 1c — transport-polymorphic typed channels.
//!
//! Substrate plumbing that makes the user-visible
//! `:wat::kernel::Sender<T>` / `:wat::kernel::Receiver<T>` abstraction
//! uniform across runtime tiers (per
//! `docs/arc/2026/05/170-program-entry-points/TIERS.md`):
//!
//! - **Tier 1 — threads.** Crossbeam channels carry typed `Value`s
//!   in-memory. No encoding step.
//! - **Tier 2 — processes.** Linux pipes carry EDN-encoded bytes;
//!   the substrate encodes typed Values on send and decodes on
//!   recv. The user-facing send / recv signature is identical to
//!   tier 1; transport is substrate-internal.
//! - **Tier 3 — remote programs (future).** Sockets carry EDN-
//!   encoded bytes via the same `PipeFd`-style wrapper.
//!
//! ## Implementation choice
//!
//! `BRIEF-SLICE-1C.md` enumerated three options for transport
//! polymorphism (separate Value variants, transport-polymorphic
//! Value with internal enum, multimethod dispatch). This module
//! ships **Option B** — one `Value::wat__kernel__Sender` /
//! `Value::wat__kernel__Receiver` variant, with the per-transport
//! payload carried by an internal [`SenderInner`] / [`ReceiverInner`]
//! enum.
//!
//! Reasoning:
//! - Option A (separate variants) doubles the variant surface and
//!   forces every send / recv / select / drop callsite to dispatch
//!   on two Value variants. The wat-side `:wat::kernel::Sender`
//!   typealias couldn't unify both transports without a polymorphic
//!   union (which the substrate doesn't have).
//! - Option C (multimethod via arc 146) is structurally over-
//!   engineered for binary internal dispatch on a single Value
//!   variant.
//! - Option B unifies the Value variant; the inner enum dispatch
//!   is local to send / recv impls. Existing crossbeam call sites
//!   that pattern-matched on `Value::crossbeam_channel__Sender(_)`
//!   migrate to `Value::wat__kernel__Sender(_)` and unwrap the
//!   inner enum where they actually call `.send()` / `.recv()`.
//!   `feedback_capability_carrier.md` shape — extend the existing
//!   entity rather than minting parallel ones.
//!
//! ## Wire protocol (tier 2)
//!
//! Per `project_pipe_protocol.md`: line-delimited EDN. One typed
//! `Value` per line. The encoder calls
//! [`crate::edn_shim::value_to_edn_with`] for the typed Value, then
//! [`wat_edn::write`] to bytes, then appends `'\n'`. The decoder
//! reads via [`crate::io::WatReader::read_line`] (which strips
//! trailing `\n`/`\r`) and parses with [`crate::edn_shim::read_edn`].
//! Same convention `:wat::kernel::process-send` / `process-recv`
//! already use over the legacy byte-pipe path.
//!
//! ## Error semantics (tier 2)
//!
//! - Sender side: a write to a pipe whose reader has gone away
//!   surfaces as a Rust-level `RuntimeError::MalformedForm` from
//!   [`crate::io::PipeWriter::write_all`]. The send wrapper maps
//!   that to a wat-level `Result.Err(ChannelDisconnected)` —
//!   same shape crossbeam-disconnect produces, so the user code
//!   can match one error pattern regardless of transport.
//! - Receiver side: pipe EOF (writer end closed) maps to wat-
//!   level `Ok(:None)` — clean shutdown. EDN parse failure on a
//!   non-empty line maps to a `RuntimeError` raised via the
//!   primitive, matching `:wat::kernel::process-recv`'s
//!   pre-existing behaviour for malformed input.

use crate::io::{WatReader, WatWriter};
use crate::span::Span;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Transport-polymorphic Sender backing for
/// `Value::wat__kernel__Sender`.
///
/// `Crossbeam` carries a tier-1 in-memory channel (no encoding).
/// `PipeFd` wraps a writer fd with EDN encoding on send.
///
/// Arc 170 slice 3 Gap B — each variant carries a `closed` flag
/// (`AtomicBool`) so `:wat::kernel::Sender/close` can signal EOF
/// on the send side without dropping the Sender Value. Interior
/// mutability via `AtomicBool` is permitted under zero-Mutex
/// doctrine (ZERO-MUTEX.md § "Honest caveats"). The `Arc<SenderInner>`
/// wrapping remains immutable from Rust's ownership perspective;
/// only the flag's value changes.
#[derive(Debug)]
pub enum SenderInner {
    /// Tier 1 — crossbeam in-memory channel. Same Arc the legacy
    /// `Value::crossbeam_channel__Sender` carried.
    Crossbeam {
        sender: crossbeam_channel::Sender<crate::runtime::Value>,
        /// Arc 170 slice 3 Gap B — set by `Sender/close`; checked
        /// by `typed_send` before each send attempt.
        closed: AtomicBool,
    },
    /// Tier 2 — linux-fd pipe with EDN encoding on send. The
    /// inner `Arc<dyn WatWriter>` is the same shape `Process.stdin`
    /// has carried since arc 103 (PipeWriter from an OwnedFd).
    PipeFd {
        writer: Arc<dyn WatWriter>,
        /// Arc 170 slice 3 Gap B — set by `Sender/close`; checked
        /// by `typed_send` before each send attempt. For PipeFd,
        /// `Sender/close` also calls `writer.close()` which releases
        /// the underlying fd so the peer reader sees EOF.
        closed: AtomicBool,
    },
}

/// Transport-polymorphic Receiver backing for
/// `Value::wat__kernel__Receiver`.
#[derive(Debug)]
pub enum ReceiverInner {
    /// Tier 1 — crossbeam in-memory channel.
    Crossbeam(crossbeam_channel::Receiver<crate::runtime::Value>),
    /// Tier 2 — linux-fd pipe with line-delimited EDN decoding on
    /// recv. The inner `Arc<dyn WatReader>` is the same shape
    /// `Process.stdout` has carried since arc 103 (PipeReader from
    /// an OwnedFd).
    PipeFd(Arc<dyn WatReader>),
}

/// Ergonomic constructor for a tier-1 (crossbeam-backed) Sender
/// `Value`. Replacement for the legacy
/// `Value::crossbeam_channel__Sender(Arc::new(tx))` pattern.
pub fn sender_from_crossbeam(
    tx: crossbeam_channel::Sender<crate::runtime::Value>,
) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Sender(Arc::new(SenderInner::Crossbeam {
        sender: tx,
        closed: AtomicBool::new(false),
    }))
}

/// Ergonomic constructor for a tier-1 (crossbeam-backed) Receiver
/// `Value`.
pub fn receiver_from_crossbeam(
    rx: crossbeam_channel::Receiver<crate::runtime::Value>,
) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Receiver(Arc::new(ReceiverInner::Crossbeam(rx)))
}

/// Ergonomic constructor for a tier-2 (pipe-fd) Sender `Value`.
/// The wrapped writer encodes typed `Value`s as line-delimited EDN
/// on each send.
pub fn sender_from_pipe(writer: Arc<dyn WatWriter>) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Sender(Arc::new(SenderInner::PipeFd {
        writer,
        closed: AtomicBool::new(false),
    }))
}

/// Ergonomic constructor for a tier-2 (pipe-fd) Receiver `Value`.
pub fn receiver_from_pipe(reader: Arc<dyn WatReader>) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Receiver(Arc::new(ReceiverInner::PipeFd(reader)))
}

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
        SenderInner::Crossbeam { sender: tx, closed } => {
            // Arc 170 slice 3 Gap B — check closed flag before
            // attempting transport send. Acquire ordering pairs with
            // the SeqCst store in sender_close so this thread sees
            // the flag update from any concurrent close call.
            if closed.load(Ordering::Acquire) {
                return SendOutcome::Disconnected;
            }
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
) -> Result<(), crate::runtime::RuntimeError> {
    match sender {
        SenderInner::Crossbeam { closed, .. } => {
            // SeqCst store ensures all threads see the flag; Acquire
            // load in typed_send pairs with this.
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
        ReceiverInner::Crossbeam(rx) => {
            let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
            match shutdown_rx {
                Some(srx) => {
                    crossbeam_channel::select! {
                        recv(rx) -> msg => match msg {
                            Ok(v) => RecvOutcome::Value(v),
                            Err(_) => RecvOutcome::Disconnected,
                        },
                        recv(srx) -> _ => RecvOutcome::Shutdown,
                    }
                }
                None => {
                    // Bootstrap pre-init or test bypass — fall back to bare recv.
                    // Should not happen in production paths; init_shutdown_signal()
                    // runs before any wat code can execute (freeze.rs:233).
                    match rx.recv() {
                        Ok(v) => RecvOutcome::Value(v),
                        Err(_) => RecvOutcome::Disconnected,
                    }
                }
            }
        }
        ReceiverInner::PipeFd(reader) => match reader.read_line(span) {
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
        },
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
/// - Tier 2 (PipeFd): NOT YET IMPLEMENTED — pipe fds are
///   blocking by default, and the substrate doesn't currently
///   set O_NONBLOCK on Process pipes. Callers that reach for
///   `try-recv` on a pipe-backed Receiver get `Disconnected` as
///   a stand-in (matches the crossbeam empty / disconnected
///   collapse). Surface as honest delta if a real consumer
///   demands non-blocking pipe recv.
pub fn typed_try_recv(
    receiver: &ReceiverInner,
    _types: Option<&crate::types::TypeEnv>,
    _span: Span,
) -> RecvOutcome {
    match receiver {
        ReceiverInner::Crossbeam(rx) => {
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
            match rx.try_recv() {
                Ok(v) => RecvOutcome::Value(v),
                Err(_) => RecvOutcome::Disconnected,
            }
        }
        ReceiverInner::PipeFd(_) => RecvOutcome::Disconnected,
    }
}

/// Helper for `:wat::kernel::select` — extracts the underlying
/// crossbeam Receiver if the inner is `Crossbeam`. Returns `None`
/// for `PipeFd` (select is crossbeam-only today; piped channels
/// would need an epoll/poll integration that's substrate work
/// for a follow-up arc).
pub fn try_as_crossbeam_receiver(
    receiver: &ReceiverInner,
) -> Option<&crossbeam_channel::Receiver<crate::runtime::Value>> {
    match receiver {
        ReceiverInner::Crossbeam(rx) => Some(rx),
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
/// — same shape `:wat::kernel::make-bounded-channel` returns for
/// the tier-1 case. `T` is phantom at the runtime layer; the
/// type checker enforces homogeneity per FOUNDATION.
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
) -> Result<(crate::runtime::Value, crate::runtime::Value), crate::runtime::RuntimeError> {
    let (read_fd, write_fd) = crate::fork::make_pipe(op)?;
    let writer: Arc<dyn WatWriter> = Arc::new(crate::io::PipeWriter::from_owned_fd(write_fd));
    let reader: Arc<dyn WatReader> = Arc::new(crate::io::PipeReader::from_owned_fd(read_fd));
    Ok((sender_from_pipe(writer), receiver_from_pipe(reader)))
}

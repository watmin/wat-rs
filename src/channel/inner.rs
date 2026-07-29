//! The transport-polymorphic seam: `SenderInner` / `ReceiverInner` and the
//! four constructors. Lifted verbatim from `src/typed_channel.rs` at Stone 6.1;
//! behavior identical.

use crate::io::WatReader;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Transport-polymorphic Sender backing for
/// `Value::wat__kernel__Sender`.
///
/// `Comms` carries a tier-1 in-memory channel backed by
/// `crate::comms::thread::Sender<Value>` (cascade-aware, depth-1).
/// `PipeFd` wraps a writer fd with EDN encoding on send.
///
/// Arc 170 slice 3 Gap B / Arc 214 Stone 5.1 — each tier-1 variant
/// carries a `closed` flag (`AtomicBool`) so `crate::channel::sender_close`
/// can signal EOF on the send side without dropping the Sender Value.
/// Interior mutability via `AtomicBool` is permitted under zero-Mutex
/// doctrine (ZERO-MUTEX.md § "Honest caveats"). The `Arc<SenderInner>`
/// wrapping remains immutable from Rust's ownership perspective;
/// only the flag's value changes.
///
/// Arc 214 Stone 5.1 HARD CUT — the `Crossbeam` variant is deleted.
/// `Comms` is the sole tier-1 backing. All callers construct via
/// `sender_from_comms`.
#[derive(Debug)]
pub enum SenderInner {
    /// Tier 1 — comms::thread in-memory channel (cascade-aware, depth-1).
    /// Replaces the retired `Crossbeam` variant (Arc 214 Stone 5.1).
    Comms {
        sender: crate::comms::thread::Sender<crate::runtime::Value>,
        /// Arc 170 slice 3 Gap B — set by `crate::channel::sender_close`;
        /// checked by `typed_send` before each send attempt.
        closed: AtomicBool,
    },
}

/// Transport-polymorphic Receiver backing for
/// `Value::wat__kernel__Receiver`.
///
/// Arc 214 Stone 5.1 HARD CUT — the `Crossbeam` variant is deleted.
/// `Comms` is the sole tier-1 backing. All callers construct via
/// `receiver_from_comms`.
#[derive(Debug)]
pub enum ReceiverInner {
    /// Tier 1 — comms::thread in-memory channel (cascade-aware, depth-1).
    /// Replaces the retired `Crossbeam` variant (Arc 214 Stone 5.1).
    Comms(crate::comms::thread::Receiver<crate::runtime::Value>),
    /// Tier 2 — linux-fd pipe with line-delimited EDN decoding on
    /// recv. The inner `Arc<dyn WatReader>` is the same shape
    /// `Process.stdout` has carried since arc 103 (PipeReader from
    /// an OwnedFd).
    PipeFd(Arc<dyn WatReader>),
}

/// Ergonomic constructor for a tier-1 (comms::thread-backed) Sender
/// `Value`. Arc 214 Stone 5.1 — replaces `sender_from_crossbeam`.
pub fn sender_from_comms(
    tx: crate::comms::thread::Sender<crate::runtime::Value>,
) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Sender(Arc::new(SenderInner::Comms {
        sender: tx,
        closed: AtomicBool::new(false),
    }))
}

/// Ergonomic constructor for a tier-1 (comms::thread-backed) Receiver
/// `Value`. Arc 214 Stone 5.1 — replaces `receiver_from_crossbeam`.
pub fn receiver_from_comms(
    rx: crate::comms::thread::Receiver<crate::runtime::Value>,
) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Receiver(Arc::new(ReceiverInner::Comms(rx)))
}

/// Ergonomic constructor for a tier-2 (pipe-fd) Receiver `Value`.
pub fn receiver_from_pipe(reader: Arc<dyn WatReader>) -> crate::runtime::Value {
    crate::runtime::Value::wat__kernel__Receiver(Arc::new(ReceiverInner::PipeFd(reader)))
}

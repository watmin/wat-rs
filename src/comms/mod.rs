//! # Comms layer — substrate-internal tier primitives
//!
//! Layer 0a of arc 214's concurrency toolkit. This module defines the
//! tier-agnostic abstractions (HolonRepresentable wire form, CommSender /
//! CommReceiver traits, error types, SelectOutcome) shared by the thread
//! tier (`comms::thread`) and process tier (`comms::process`) implementations.
//!
//! ## Cascade contract (LOAD-BEARING)
//!
//! Every blocking method on tier-specific Receivers + Selects MUST wake on
//! substrate shutdown:
//!
//! - Thread tier: `crossbeam_channel::select! { recv(data), recv(SHUTDOWN_RX) }`
//!   — substrate's shutdown cascade signals via crossbeam channel; tier recv
//!   includes this in its select arm.
//! - Process tier: `io_uring` multi-arm submission on [data_fd, broadcast_fd]
//!   — substrate's broadcast pipe acts as the wake signal; first completion
//!   wins.
//!
//! Callers cannot bypass the cascade because tier wrappers hide the underlying
//! mechanism. Bare `crossbeam_channel::*` and bare `libc::pipe/read/write/
//! poll/epoll/io_uring_*` are unreachable outside the tier wrapper modules
//! (Slice 6 structural wall).
//!
//! ## Audience
//!
//! - **Substrate authors** (building brackets, services, kernel-layer dispatch)
//!   use this module directly via `crate::comms::thread::*` / `crate::comms::
//!   process::*`.
//! - **User code** does NOT touch this layer; uses peer-oriented `:wat::kernel::*`
//!   verbs (Slice 4) that internally dispatch to the right tier.

// ─── Wire form trait ────────────────────────────────────────────────────────

/// Universal wire form for cross-boundary types. Anything that crosses a
/// process or remote tier boundary must roundtrip through HolonAST (substrate's
/// universal "Any" form per arc 057+ project_holon_universal_ast).
///
/// Thread-tier (in-process) channels can also use HolonRepresentable types,
/// but pass T directly via crossbeam (no serialization roundtrip).
///
/// Per project_holon_universal_ast (the strange loop closing 2026-05-19): HolonAST
/// was minted for VSA encoding (arc 057), became universal AST (arc 143 signature
/// reflection, arc 201 type reflection), and is NOW also the universal comms wire
/// form.
///
/// # Blanket impl decision
///
/// A blanket impl `impl<T> HolonRepresentable for T where T: Into<HolonAST> + ...`
/// is NOT included here. Reason: `Into<HolonAST>` consumes self, while
/// `HolonRepresentable::to_holon_ast` takes `&self`. A blanket form would require
/// `T: Clone` overhead at every send (clone-then-convert). The cost is silent and
/// invisible at call sites. Manual `impl HolonRepresentable for T` per
/// substrate-internal type is the honest form — each impl documents the conversion
/// explicitly, and no hidden clone tax exists at send boundaries. Future arc may
/// revisit if a clean zero-cost blanket pattern surfaces (e.g., `for<'a>
/// HolonAST: From<&'a T>` reference-style conversion without consume).
pub trait HolonRepresentable: Send + 'static {
    fn to_holon_ast(&self) -> holon::HolonAST;
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized;
}

// ─── Tier-agnostic sender / receiver traits ─────────────────────────────────

/// Tier-agnostic send endpoint. Implemented by `comms::thread::Sender<T>` (Slice 2)
/// and `comms::process::Sender<T>` (Slice 3). Enables tier-agnostic generic
/// functions for brackets + services that work across both transport layers.
pub trait CommSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
    /// Signal end-of-stream from this sender. Consumes self so the endpoint
    /// is gone after close. Other cloned `Sender` handles (if any) remain
    /// valid. Peer receivers will see `RecvError` / `TryRecvError::Disconnected`
    /// on their next operation only after ALL `Sender` clones close.
    fn close(self) -> Result<(), CloseError>;
}

/// Tier-agnostic receive endpoint. Implemented by `comms::thread::Receiver<T>` (Slice 2)
/// and `comms::process::Receiver<T>` (Slice 3). Enables tier-agnostic generic
/// functions for brackets + services that work across both transport layers.
///
/// Every blocking method on tier-specific implementations MUST wake on
/// substrate shutdown (cascade contract documented in this module's top-level doc).
pub trait CommReceiver<T> {
    /// Cascade-aware blocking recv. Wakes on substrate shutdown (returns
    /// `Err(RecvError)` when all senders are dropped or the substrate signals
    /// shutdown). Tier implementations wire the shutdown signal automatically —
    /// callers cannot bypass the cascade.
    fn recv(&self) -> Result<T, RecvError>;
    /// Non-blocking recv. Returns `Err(TryRecvError::Empty)` when no value is
    /// currently available; `Err(TryRecvError::Disconnected)` when all senders
    /// have dropped. Cascade-irrelevant (does not block; shutdown does not change
    /// the result).
    fn try_recv(&self) -> Result<T, TryRecvError>;
    /// Number of values currently queued in the channel awaiting recv.
    /// Non-blocking; cascade-irrelevant. Useful for capacity-tracking callers
    /// (e.g., `wat::kernel::HandlePool` checking for orphaned handles).
    fn len(&self) -> usize;
    /// Signal end-of-stream from this receiver. Consumes self so the endpoint
    /// is gone after close. Other cloned `Receiver` handles (if any) remain
    /// valid. Peer senders will see `SendError` on their next `send` only after
    /// ALL `Receiver` clones close.
    fn close(self) -> Result<(), CloseError>;
}

// ─── Error types ─────────────────────────────────────────────────────────────

/// Send failed: receiver was dropped or substrate shut down.
///
/// Holds the unsent value so the caller can inspect or recover it.
/// Shape matches `crossbeam_channel::SendError<T>` for ergonomic familiarity.
#[derive(Debug)]
pub struct SendError<T>(pub T);

/// Recv failed: all senders dropped or substrate shut down.
///
/// Shape matches `crossbeam_channel::RecvError` for ergonomic familiarity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RecvError;

/// Non-blocking recv result. Callers MUST distinguish the two variants:
/// `Empty` means "no value now; retry later may succeed";
/// `Disconnected` means "no value now and no value ever; channel permanently closed".
/// The distinction drives retry-vs-bail-out logic at every `try_recv` site.
///
/// Shape matches `crossbeam_channel::TryRecvError` for ergonomic familiarity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TryRecvError {
    /// Channel is empty; no value currently available. May become non-empty later.
    Empty,
    /// All senders dropped; channel will never produce another value.
    Disconnected,
}

/// Close failed (rare; e.g., FD already closed at the OS level).
///
/// Field is private so callers cannot inject arbitrary close errors;
/// only tier-specific `Sender` / `Receiver` impls construct via `new()`.
#[derive(Debug)]
pub struct CloseError(String);

impl CloseError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

/// HolonAST roundtrip failure during wire serialization/deserialization.
///
/// Produced by `HolonRepresentable::from_holon_ast` when the incoming AST
/// does not match the expected variant or carry a valid payload.
///
/// Field is private so only `HolonRepresentable` impls construct via `new()`;
/// callers cannot inject arbitrary wire errors.
#[derive(Debug)]
pub struct WireError(String);

impl WireError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    pub fn message(&self) -> &str {
        &self.0
    }
}

// ─── Tier modules ────────────────────────────────────────────────────────────

/// Thread tier: in-process comms via crossbeam_channel. Cascade-aware.
/// Substrate-internal; user code uses `:wat::kernel::*` verbs (Slice 4).
pub mod thread;

// ─── Select outcome ───────────────────────────────────────────────────────────

/// User-assigned index of a receiver registered with a tier-specific `Select`.
///
/// Newtype over `usize` so `SelectOutcome::Recv { index: ReceiverIndex(_), .. }`
/// cannot be confused with a count, capacity, or offset. The index is what
/// the caller passed when registering the receiver — it identifies WHICH
/// receiver fired, not HOW MANY fired.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ReceiverIndex(pub usize);

/// Result of a cascade-aware fan-in select over multiple receivers.
///
/// Tier-specific `Select` types (Slice 2: `comms::thread::Select`,
/// Slice 3: `comms::process::Select`) return this enum so callers
/// handle substrate-shutdown uniformly regardless of which tier fired.
#[derive(Debug)]
pub enum SelectOutcome<T> {
    /// One of the registered receivers fired.
    Recv {
        /// Index of the receiver that fired (as registered by the caller).
        index: ReceiverIndex,
        /// Recv result: `Ok(value)` on success; `Err(RecvError)` when the
        /// fired receiver's channel is disconnected.
        result: Result<T, RecvError>,
    },
    /// Substrate shutdown fired before any data receiver. Caller should unwind.
    Shutdown,
}

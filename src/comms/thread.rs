//! # Thread tier — in-process comms via crossbeam_channel
//!
//! Layer 0a tier implementation per arc 214's `DESIGN.md`. Builds on the
//! Slice 1 traits (`crate::comms::{CommSender, CommReceiver, SelectOutcome,
//! ReceiverIndex, SendError, RecvError, TryRecvError, CloseError}`) with
//! `crossbeam_channel` underneath.
//!
//! ## Cascade contract (LOAD-BEARING)
//!
//! Every blocking method auto-wires the substrate's shutdown signal:
//! - `Receiver::recv()` uses `crossbeam_channel::select! { recv(data),
//!   recv(SHUTDOWN_RX) }` — on shutdown, recv returns `Err(RecvError)`
//!   instead of hanging indefinitely.
//! - `Select::select()` registers `SHUTDOWN_RX` as an internal arm —
//!   on shutdown, returns `SelectOutcome::Shutdown` regardless of which
//!   user receivers are pending.
//!
//! Bootstrap fallback: when `SHUTDOWN_RX` is uninitialized (pre-init or
//! test bypass), blocking methods fall back to bare crossbeam recv.
//! Production paths always have `SHUTDOWN_RX` initialized by
//! `freeze.rs:233` before any wat code executes.
//!
//! ## Audience
//!
//! Substrate-internal Rust code (brackets, services, kernel layer
//! dispatch). User code does NOT touch this tier — it uses peer-oriented
//! `:wat::kernel::*` verbs (Slice 4) that internally dispatch here.

use crate::comms::{
    CloseError, CommReceiver, CommSender, ReceiverIndex, RecvError, SelectOutcome, SendError,
    TryRecvError,
};

// ─── Sender ──────────────────────────────────────────────────────────────────

/// Thread-tier send endpoint. Wraps `crossbeam_channel::Sender<T>` with
/// the tier-agnostic `CommSender` trait surface. Private inner field
/// prevents bare crossbeam access from outside the tier.
#[derive(Debug)]
pub struct Sender<T> {
    inner: crossbeam_channel::Sender<T>,
}

impl<T> Sender<T> {
    /// Send a value to the channel. Returns `Err(SendError(value))` if
    /// all receivers have been dropped (cf. `crossbeam::SendError`).
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.inner
            .send(value)
            .map_err(|crossbeam_channel::SendError(v)| SendError(v))
    }

    /// Signal end-of-stream from this sender. Consumes self so the endpoint
    /// is gone after close. Other cloned `Sender` handles (if any) remain
    /// valid. Peer receivers will see `RecvError` / `TryRecvError::Disconnected`
    /// on their next operation only after ALL `Sender` clones close.
    ///
    /// Thread-tier close always succeeds (crossbeam channels don't fail at
    /// close — the Drop impl handles cleanup). Returns `Ok(())` after
    /// consuming self.
    pub fn close(self) -> Result<(), CloseError> {
        // self is dropped at end of scope; crossbeam decrements its
        // internal sender count; when count hits zero, receivers see
        // Disconnected. No fallible operation; always Ok.
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Send + 'static> CommSender<T> for Sender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>> {
        Sender::send(self, value)
    }

    fn close(self) -> Result<(), CloseError> {
        Sender::close(self)
    }
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// Thread-tier receive endpoint. Wraps `crossbeam_channel::Receiver<T>`
/// with cascade-aware blocking recv. Private inner field prevents bare
/// crossbeam access from outside the tier.
#[derive(Debug)]
pub struct Receiver<T> {
    inner: crossbeam_channel::Receiver<T>,
}

impl<T> Receiver<T> {
    /// Cascade-aware blocking recv. Routes through `SHUTDOWN_RX` via
    /// `crossbeam::select! { recv(data), recv(SHUTDOWN_RX) }`. When
    /// substrate shutdown fires, parked recvs wake with `Err(RecvError)`
    /// instead of hanging indefinitely.
    ///
    /// Bootstrap fallback: when `SHUTDOWN_RX` is `None`, falls back to
    /// bare crossbeam recv. Production paths always have SHUTDOWN_RX
    /// initialized before wat code executes.
    pub fn recv(&self) -> Result<T, RecvError> {
        // rune:forge(escape) — SHUTDOWN_RX is the substrate cascade signal;
        // global access is the cascade contract (module-level doc § Cascade contract).
        let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();
        match shutdown_rx {
            Some(srx) => {
                crossbeam_channel::select! {
                    recv(&self.inner) -> msg => msg.map_err(|_| RecvError),
                    recv(srx) -> _ => Err(RecvError),
                }
            }
            None => self.inner.recv().map_err(|_| RecvError),
        }
    }

    /// Non-blocking recv. Returns `Err(TryRecvError::Empty)` when no value
    /// is currently available; `Err(TryRecvError::Disconnected)` when all
    /// senders have dropped. Cascade-irrelevant (does not block).
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.inner.try_recv().map_err(|e| match e {
            crossbeam_channel::TryRecvError::Empty => TryRecvError::Empty,
            crossbeam_channel::TryRecvError::Disconnected => TryRecvError::Disconnected,
        })
    }

    /// Number of values currently queued in the channel awaiting recv.
    /// Non-blocking; cascade-irrelevant. Trivial passthrough to
    /// `crossbeam::Receiver::len`. Useful for capacity-tracking callers
    /// (e.g., `HandlePool` checking for orphaned handles).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Signal end-of-stream from this receiver. Consumes self so the
    /// endpoint is gone after close. Other cloned `Receiver` handles (if
    /// any) remain valid. Peer senders will see `SendError` on their next
    /// `send` only after ALL `Receiver` clones close. Thread-tier close
    /// always succeeds.
    pub fn close(self) -> Result<(), CloseError> {
        // Drop happens at end of scope; crossbeam decrements receiver count.
        // When count hits zero, senders see SendError. Always Ok.
        Ok(())
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Send + 'static> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        Receiver::recv(self)
    }

    fn try_recv(&self) -> Result<T, TryRecvError> {
        Receiver::try_recv(self)
    }

    fn len(&self) -> usize {
        Receiver::len(self)
    }

    fn close(self) -> Result<(), CloseError> {
        Receiver::close(self)
    }
}

// ─── Select ──────────────────────────────────────────────────────────────────

/// Cascade-aware fan-in over multiple thread-tier receivers. Auto-registers
/// `SHUTDOWN_RX` as an internal arm; user-registered receivers get
/// `ReceiverIndex`es in registration order.
///
/// When `select()` fires, the shutdown arm wins iff substrate shutdown
/// signaled (returns `SelectOutcome::Shutdown`); otherwise the fired
/// user receiver's index + recv result is returned.
pub struct Select<'a, T: Send + 'static> {
    inner: crossbeam_channel::Select<'a>,
    /// Crossbeam-internal arm index for SHUTDOWN_RX (if registered).
    /// `None` only in bootstrap (SHUTDOWN_RX uninitialized).
    shutdown_arm: Option<usize>,
    /// Direct-index lookup table: `crossbeam_to_user[arm_idx]` returns the
    /// `Some(user_pos)` mapping for that arm, or `None` for the shutdown arm.
    /// O(1) lookup at `select()` time; sized to fit the largest crossbeam-arm-idx
    /// registered. Built incrementally as `recv()` adds user arms.
    crossbeam_to_user: Vec<Option<usize>>,
    /// User-registered receivers indexed by user_pos (registration order).
    /// `select()` returns `ReceiverIndex(user_pos)` so callers see registration
    /// order, independent of the crossbeam-internal arm assignment.
    user_arms: Vec<&'a Receiver<T>>,
}

impl<'a, T: Send + 'static> Select<'a, T> {
    /// Construct a new cascade-aware Select. Auto-registers `SHUTDOWN_RX`
    /// internally so callers don't manually wire shutdown into every Select.
    pub fn new() -> Self {
        let mut inner = crossbeam_channel::Select::new();
        // rune:forge(escape) — SHUTDOWN_RX is the substrate cascade signal;
        // global access is the cascade contract (module-level doc § Cascade contract).
        // Register SHUTDOWN_RX first (arm 0) when available so the cascade
        // is always present regardless of how many user arms are added later.
        let shutdown_arm = crate::runtime::SHUTDOWN_RX
            .get()
            .map(|srx| inner.recv(srx));
        // Seed crossbeam_to_user with `None` at the shutdown arm's slot so the
        // table is dense from arm 0 forward; user arms fill from arm 1+.
        let mut crossbeam_to_user = Vec::new();
        if let Some(sa) = shutdown_arm {
            crossbeam_to_user.resize(sa + 1, None);
        }
        Self {
            inner,
            shutdown_arm,
            crossbeam_to_user,
            user_arms: Vec::new(),
        }
    }

    /// Register a receiver. Returns the `ReceiverIndex` the caller will
    /// see in `SelectOutcome::Recv { index, .. }` when this receiver fires.
    /// Index is the registration order (0 for first registered, 1 for
    /// second, etc.) — independent of the crossbeam-internal arm index.
    pub fn recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex {
        let arm_idx = self.inner.recv(&rx.inner);
        let user_pos = self.user_arms.len();
        // Grow the lookup table to cover this arm_idx (crossbeam assigns
        // arms in ascending order; new arm_idx ≥ current table len).
        if self.crossbeam_to_user.len() <= arm_idx {
            self.crossbeam_to_user.resize(arm_idx + 1, None);
        }
        self.crossbeam_to_user[arm_idx] = Some(user_pos);
        self.user_arms.push(rx);
        ReceiverIndex(user_pos)
    }

    /// Block until any registered receiver fires OR substrate shutdown
    /// signals. Returns the outcome — `Recv { index, result }` for a user
    /// receiver firing, `Shutdown` when the cascade fires.
    pub fn select(&mut self) -> SelectOutcome<T> {
        let selected_op = self.inner.select();
        let arm_idx = selected_op.index();

        // Shutdown arm takes priority: if the substrate signaled shutdown,
        // consume the operation and return Shutdown so callers unwind cleanly.
        if self.shutdown_arm == Some(arm_idx) {
            // rune:forge(escape) — SHUTDOWN_RX is the substrate cascade signal;
            // global access is the cascade contract (module-level doc § Cascade contract).
            let srx = crate::runtime::SHUTDOWN_RX
                .get()
                .expect("shutdown_arm was Some so SHUTDOWN_RX must be initialized");
            // Consume the SelectedOperation — crossbeam requires this to
            // avoid panicking on drop.
            let _ = selected_op.recv(srx);
            return SelectOutcome::Shutdown;
        }

        // O(1) lookup: crossbeam_to_user[arm_idx] gives the user position
        // directly. Built incrementally as recv() registered each arm.
        let user_pos = self.crossbeam_to_user[arm_idx]
            .expect("registered receiver fired but not in crossbeam_to_user — internal bookkeeping bug");
        let fired_rx = self.user_arms[user_pos];

        let result = selected_op.recv(&fired_rx.inner).map_err(|_| RecvError);
        SelectOutcome::Recv {
            index: ReceiverIndex(user_pos),
            result,
        }
    }
}

impl<'a, T: Send + 'static> Default for Select<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Factories ───────────────────────────────────────────────────────────────

/// Create an unbounded thread-tier channel pair. Both endpoints are
/// cascade-aware (Receiver's recv wakes on shutdown). Senders never block
/// on `send` (unbounded queue absorbs all values). Use `bounded` when
/// back-pressure is required.
pub fn pair<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (Sender { inner: tx }, Receiver { inner: rx })
}

/// Create a bounded thread-tier channel pair with the given capacity.
/// Senders block on `send` when the channel is full, providing back-pressure.
/// Cascade-on-send for blocking sends is future arc work; cascade on recv
/// is already wired.
pub fn bounded<T: Send + 'static>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(capacity);
    (Sender { inner: tx }, Receiver { inner: rx })
}

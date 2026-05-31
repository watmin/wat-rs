//! # Thread tier — in-process comms via crossbeam_channel
//!
//! Layer 0a tier implementation per arc 214 (the comms-layer redesign;
//! full design at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! Builds on the Slice 1 traits (`crate::comms::{CommSender, CommReceiver,
//! SelectOutcome, ReceiverIndex, SendError, RecvError, TryRecvError}`)
//! with `crossbeam_channel` underneath.
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
//! ## Mini-TCP at depth 1 (THE only pattern)
//!
//! Every channel constructed by `pair()` has capacity 1. `send` blocks
//! when one value is queued; `recv` drains the buffer and unblocks the
//! sender. There is no `bounded(N)` factory (retired by four-questions;
//! see DESIGN § "Slice 2 forward-correction").
//!
//! Depth-1 is the MECHANISM that makes the mini-TCP usage DISCIPLINE
//! organic under load. The discipline (per `docs/ZERO-MUTEX.md` §
//! "Mini-TCP via paired channels" line 252+): each send pairs with a
//! recv before the next send fires — an ack on a separate pair, or the
//! next value on the same pair. The substrate doesn't enforce the
//! pairing site-by-site (multiple senders can saturate the buffer at
//! the same depth), but capacity-1 makes producers that try to outpace
//! consumers block immediately rather than queuing up and amplifying
//! drift. The lock-step breathes with system load.
//!
//! The process tier uses Linux anonymous pipes (~65536-byte kernel buffer;
//! `PIPE_BUF` = 4096 is the per-write POSIX-atomicity threshold, NOT the
//! pipe capacity). Linux blocks writers when the pipe buffer fills; the
//! unit differs from the thread tier (bytes vs frame-count) but the
//! backpressure shape is the same: substrate refuses to absorb work
//! the consumer can't keep up with. The process tier is NOT capacity-1
//! — it holds many frames before blocking.
//!
//! Every load-bearing pattern this substrate ships (arc 119 ack-tx,
//! defservice Request/Reply, Counter actor, dispatch loops) operates
//! at this depth. The trading-lab convergence (pre-wat-rs origin)
//! proved N > 1 produces massive perf hits + entire categories of
//! problems; depth-1 is dynamic, predictable, organic. The substrate
//! makes the wrong shape unavailable.
//!
//! ## Audience
//!
//! Substrate-internal Rust code (brackets, services, kernel layer
//! dispatch). User code does NOT touch this tier — it uses peer-oriented
//! `:wat::kernel::*` verbs (Slice 4) that internally dispatch here.

use crate::comms::{
    CommReceiver, CommSender, ReceiverIndex, RecvError, SelectOutcome, SendError, TryRecvError,
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
    /// Infallible: self is dropped at end of scope; crossbeam decrements its
    /// internal sender count; when count hits zero, receivers see Disconnected.
    /// No fallible operation; move semantics make double-close a compile error.
    pub fn close(self) {
        // Drop happens at end of scope.
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

    fn close(self) {
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
        // rune:sequi(ambient-context) — SHUTDOWN_RX is the substrate cascade signal; explicit threading would bloat every recv signature in the codebase
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
    /// `send` only after ALL `Receiver` clones close.
    ///
    /// Infallible: Drop decrements receiver count; when count hits zero,
    /// senders see SendError. Move semantics make double-close a compile error.
    pub fn close(self) {
        // Drop happens at end of scope.
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

    fn close(self) {
        Receiver::close(self)
    }
}

// ─── Select ──────────────────────────────────────────────────────────────────

/// Cascade-aware fan-in over multiple thread-tier receivers. Wires the
/// substrate's `SHUTDOWN_RX` shutdown signal on every `select()` call
/// (fresh read — no init-order trap).
///
/// User-registered receivers get `ReceiverIndex`es in registration order.
/// The shutdown arm has no user-facing index; it surfaces as
/// `SelectOutcome::Shutdown`.
///
/// When `select()` fires, the shutdown arm wins iff substrate shutdown
/// signaled (returns `SelectOutcome::Shutdown`); otherwise the fired
/// user receiver's index + recv result is returned.
///
/// ## Init-order safety
///
/// `select()` reads `SHUTDOWN_RX` FRESH on every call (matching the
/// `process::Select` pattern). A `Select` built before
/// `init_shutdown_signal()` is NOT permanently broken — the next
/// `select()` call picks up the now-initialized shutdown arm. This closes
/// the init-order trap that the `new()`-time registration would create.
pub struct Select<'a, T: Send + 'static> {
    /// User-registered receivers in registration order. The index
    /// into this Vec is the user-facing `ReceiverIndex`.
    user_arms: Vec<&'a Receiver<T>>,
}

impl<'a, T: Send + 'static> Select<'a, T> {
    /// Construct a new cascade-aware Select. Empty until receivers are
    /// registered via `recv`. The shutdown arm is NOT registered here —
    /// it's wired per-`select()` call so there is no init-order trap.
    pub fn new() -> Self {
        Self {
            user_arms: Vec::new(),
        }
    }

    /// Register a receiver. Returns the `ReceiverIndex` the caller will
    /// see in `SelectOutcome::Recv { index, .. }` when this receiver fires.
    /// Index is the registration order (0 for first registered, 1 for
    /// second, etc.) — independent of the crossbeam-internal arm index.
    pub fn recv(&mut self, rx: &'a Receiver<T>) -> ReceiverIndex {
        let user_pos = self.user_arms.len();
        self.user_arms.push(rx);
        ReceiverIndex(user_pos)
    }

    /// Block until any registered receiver fires OR substrate shutdown
    /// signals. Returns the outcome — `Recv { index, result }` for a user
    /// receiver firing, `Shutdown` when the cascade fires.
    ///
    /// Panics if called with zero registered receivers and no
    /// SHUTDOWN_RX initialized — crossbeam panics "cannot select with
    /// no operations" in that case. With zero user arms, this Select is
    /// only valid when SHUTDOWN_RX is initialized (cascade-only wait).
    /// Prefer registering at least one user receiver before calling.
    pub fn select(&mut self) -> SelectOutcome<T> {
        // rune:sequi(ambient-context) — SHUTDOWN_RX is the substrate cascade signal;
        // explicit threading would bloat every select() signature in the codebase.
        // Fresh read per call — no init-order trap (mirrors process::Select pattern).
        let shutdown_rx = crate::runtime::SHUTDOWN_RX.get();

        // Guard: zero user arms + no shutdown = crossbeam would panic.
        if self.user_arms.is_empty() && shutdown_rx.is_none() {
            panic!(
                "thread::Select::select() called with zero registered receivers \
                 and SHUTDOWN_RX uninitialized — crossbeam would panic; register \
                 at least one receiver or initialize the shutdown signal first"
            );
        }

        // Build a fresh crossbeam Select for this call, wiring the shutdown
        // arm first (so it has internal priority in crossbeam's selection).
        let mut inner = crossbeam_channel::Select::new();

        // Register shutdown arm first (arm 0 when present) so it has
        // crossbeam-internal priority over user arms.
        let shutdown_arm_idx: Option<usize> = shutdown_rx.map(|srx| inner.recv(srx));

        // Register user arms; crossbeam assigns ascending indices starting
        // after the shutdown arm (if any).
        let user_arm_start = shutdown_arm_idx.map_or(0, |sa| sa + 1);
        for rx in self.user_arms.iter() {
            inner.recv(&rx.inner);
        }

        let selected_op = inner.select();
        let arm_idx = selected_op.index();

        // Shutdown arm takes priority.
        if shutdown_arm_idx == Some(arm_idx) {
            let srx = shutdown_rx
                .expect("shutdown_arm_idx was Some so shutdown_rx must be initialized");
            let _ = selected_op.recv(srx);
            return SelectOutcome::Shutdown;
        }

        // User arm — map crossbeam index back to user_pos.
        let user_pos = arm_idx - user_arm_start;
        let fired_rx = self.user_arms[user_pos];
        let result = selected_op.recv(&fired_rx.inner).map_err(|_| RecvError);
        SelectOutcome::Recv {
            index: ReceiverIndex(user_pos),
            result,
        }
    }
}

// ─── Factories ───────────────────────────────────────────────────────────────

/// Construct a capacity-1 mini-TCP channel pair.
///
/// `send` blocks when the buffer holds one value; `recv` drains it.
/// Capacity is structural, not a tunable: N > 1 eliminates the
/// lock-step the substrate enforces. See module-level doc § "Mini-TCP
/// at depth 1" for the why-this-not-N + cross-references to the
/// trading-lab convergence and the four-questions verdict.
///
/// Both endpoints are cascade-aware (Receiver's recv wakes on substrate
/// shutdown).
pub fn pair<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    (Sender { inner: tx }, Receiver { inner: rx })
}

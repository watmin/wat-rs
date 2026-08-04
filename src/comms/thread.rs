//! # Thread tier — in-process comms via crossbeam_channel
//!
//! Layer 0a tier implementation per arc 214 (the comms-layer redesign;
//! full design at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! Builds on the Slice 1 traits (`crate::comms::{CommSender, CommReceiver,
//! SelectOutcome, ReceiverIndex, SendError, RecvError}`) with `crossbeam_channel` underneath.
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
    CommReceiver, CommSender, ReceiverIndex, RecvError, SelectOutcome, SendError, TrySendError,
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
            .map_err(|crossbeam_channel::SendError(v)| SendError::Disconnected(v))
    }

    /// Genuinely non-blocking send — crossbeam's native `try_send`. Returns
    /// `Err(TrySendError::Full(value))` immediately if the bounded(1) slot is
    /// already occupied, or `Err(TrySendError::Disconnected(value))` if all
    /// receivers are dropped, instead of blocking for capacity like
    /// [`Self::send`]. See `CommSender::try_send`'s doc for the arc 278 RST
    /// best-effort-broadcast rationale, and arc 278 Phase 3a
    /// (`TrySendOutcome`) for why the Full/Disconnected distinction is
    /// threaded through rather than collapsed.
    pub fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        self.inner.try_send(value).map_err(|e| match e {
            crossbeam_channel::TrySendError::Full(v) => TrySendError::Full(v),
            crossbeam_channel::TrySendError::Disconnected(v) => TrySendError::Disconnected(v),
        })
    }

    /// Signal end-of-stream from this sender. Consumes self so the endpoint
    /// is gone after close. Other cloned `Sender` handles (if any) remain
    /// valid. Peer receivers will see `RecvError::Disconnected`
    /// on their next recv only after ALL `Sender` clones close.
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

    fn try_send(&self, value: T) -> Result<(), TrySendError<T>> {
        Sender::try_send(self, value)
    }

    fn close(self) {
        Sender::close(self)
    }
}

// ─── Receiver internals ───────────────────────────────────────────────────────

/// Backing storage for `Receiver<T>`. Either a normal crossbeam channel
/// or a one-shot timer (arc 292 `:wat::kernel::after`).
///
/// `Timer` uses `Arc<OwnedMoveCell<T>>` so the msg can be *taken* exactly
/// once without requiring `T: Clone` — and ZERO-MUTEX (`docs/ZERO-MUTEX.md`
/// caveat 1: the cell's `AtomicBool` gate is sanctioned; a `Mutex` is not).
/// The `instant_rx` is a
/// `crossbeam_channel::after(d)` receiver that fires once after the
/// requested duration; we drain it (by receiving the `Instant`) to signal
/// readiness, then take the stored `T`.
enum ReceiverKind<T: Send> {
    /// Normal capacity-1 crossbeam channel (the only kind created by `pair()`).
    Channel(crossbeam_channel::Receiver<T>),
    /// One-shot timer: fires once after `duration`, delivering `msg`.
    ///
    /// `instant_rx` is a `crossbeam_channel::after(d)` receiver. After it
    /// fires, subsequent recv/select calls see Disconnected — the timer is
    /// one-shot. `msg` is taken exactly once via the OwnedMoveCell (atomic-gated, no lock).
    Timer {
        instant_rx: crossbeam_channel::Receiver<std::time::Instant>,
        msg: std::sync::Arc<crate::rust_deps::custodia::OwnedMoveCell<T>>,
    },
}

// ─── Receiver ────────────────────────────────────────────────────────────────

/// Thread-tier receive endpoint. Wraps either a `crossbeam_channel::Receiver<T>`
/// (normal channel) or a one-shot timer (arc 292) with cascade-aware blocking recv.
/// Private inner field prevents bare crossbeam access from outside the tier.
pub struct Receiver<T: Send> {
    inner: ReceiverKind<T>,
}

impl<T: Send + std::fmt::Debug> std::fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ReceiverKind::Channel(rx) => f.debug_tuple("Receiver::Channel").field(rx).finish(),
            ReceiverKind::Timer { .. } => f.debug_struct("Receiver::Timer").finish_non_exhaustive(),
        }
    }
}

impl<T: Send> Receiver<T> {
    /// Cascade-aware blocking recv. Routes through `SHUTDOWN_RX` via
    /// `crossbeam::select! { recv(data), recv(SHUTDOWN_RX) }`. When
    /// substrate shutdown fires, parked recvs wake with `Err(RecvError)`
    /// instead of hanging indefinitely.
    ///
    /// Bootstrap fallback: when `SHUTDOWN_RX` is `None`, falls back to
    /// bare crossbeam recv. Production paths always have SHUTDOWN_RX
    /// initialized before wat code executes.
    ///
    /// For the Timer variant: blocks on `instant_rx` (crossbeam `after(d)`)
    /// then takes the stored msg. One-shot: subsequent calls return
    /// `Err(Disconnected)` (both the instant_rx and the Option<T> are exhausted).
    pub fn recv(&self) -> Result<T, RecvError> {
        match &self.inner {
            ReceiverKind::Channel(ch) => {
                // rune:sequi(ambient-context) — SHUTDOWN_RX is the substrate cascade signal; explicit threading would bloat every recv signature in the codebase
                let shutdown_rx = crate::runtime::shutdown_rx();
                match shutdown_rx {
                    Some(srx) => {
                        crossbeam_channel::select! {
                            recv(ch) -> msg => msg.map_err(|_| RecvError::Disconnected),
                            recv(srx) -> _ => Err(RecvError::Shutdown),
                        }
                    }
                    None => ch.recv().map_err(|_| RecvError::Disconnected),
                }
            }
            ReceiverKind::Timer { instant_rx, msg } => {
                // Block on the timer channel (or shutdown) WITHOUT holding the msg lock.
                // crossbeam::after(d) parks on a futex — not thread::sleep.
                let shutdown_rx = crate::runtime::shutdown_rx();
                let fired = match shutdown_rx {
                    Some(srx) => crossbeam_channel::select! {
                        recv(instant_rx) -> r => r.map(|_| ()).map_err(|_| RecvError::Disconnected),
                        recv(srx) -> _ => Err(RecvError::Shutdown),
                    },
                    None => instant_rx.recv().map(|_| ()).map_err(|_| RecvError::Disconnected),
                };
                fired?;
                // Timer fired; take the msg (one-shot, atomic-gated — zero mutex).
                msg.take(":wat::kernel::after", crate::rust_caller_span!())
                    .map_err(|_| RecvError::Disconnected)
            }
        }
    }

    /// Number of values currently queued in the channel awaiting recv.
    /// Non-blocking; cascade-irrelevant. Trivial passthrough to
    /// `crossbeam::Receiver::len`. Useful for capacity-tracking callers
    /// (e.g., `HandlePool` checking for orphaned handles).
    // rune:excusare(perennial) — is_empty() withheld at the trait level for the kernel-invisible process-tier len() approximation (see CommReceiver); the thread tier's len() is exact but the trait contract is unified — adding is_empty() to one tier and not the other breaks the unified surface. Perennial per the transport model.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match &self.inner {
            ReceiverKind::Channel(ch) => ch.len(),
            ReceiverKind::Timer { instant_rx, .. } => instant_rx.len(),
        }
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

impl<T: Send> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        match &self.inner {
            ReceiverKind::Channel(rx) => Self {
                inner: ReceiverKind::Channel(rx.clone()),
            },
            ReceiverKind::Timer { instant_rx, msg } => Self {
                inner: ReceiverKind::Timer {
                    instant_rx: instant_rx.clone(),
                    msg: std::sync::Arc::clone(msg),
                },
            },
        }
    }
}

impl<T: Send + 'static> CommReceiver<T> for Receiver<T> {
    fn recv(&self) -> Result<T, RecvError> {
        Receiver::recv(self)
    }

    fn len(&self) -> usize {
        Receiver::len(self)
    }

    fn close(self) {
        Receiver::close(self)
    }

    fn reactor_class(&self) -> crate::comms::ReactorClass {
        crate::comms::ReactorClass::InMemory
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
    // rune:excusare(perennial) — Default withheld by design: an empty Select panics at select() time (no-arm footgun the comms vigilia eliminated). A Default impl would produce the exact prohibited empty value with no call-site signal that arm registration is required. Any relaxation would require removing the empty-Select guard, which would trip the comms ward (struere empty-Select finding) first.
    #[allow(clippy::new_without_default)]
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
        let shutdown_rx = crate::runtime::shutdown_rx();

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
        // For Channel arms: register the value channel directly.
        // For Timer arms: register the instant_rx (crossbeam::after(d)) — the
        //   timer fires once after the duration, delivering a std::time::Instant.
        //   The stored T is retrieved after the arm fires (see below).
        let user_arm_start = shutdown_arm_idx.map_or(0, |sa| sa + 1);
        for rx in self.user_arms.iter() {
            match &rx.inner {
                ReceiverKind::Channel(ch) => { inner.recv(ch); }
                ReceiverKind::Timer { instant_rx, .. } => { inner.recv(instant_rx); }
            }
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

        match &fired_rx.inner {
            ReceiverKind::Channel(ch) => {
                let result = selected_op.recv(ch).map_err(|_| RecvError::Disconnected);
                SelectOutcome::Recv {
                    index: ReceiverIndex(user_pos),
                    result,
                }
            }
            ReceiverKind::Timer { instant_rx, msg } => {
                // Consume the Instant to drain the timer channel.
                let _ = selected_op.recv(instant_rx);
                // Take the stored msg (one-shot, atomic-gated — zero mutex).
                let result = msg
                    .take(":wat::kernel::after", crate::rust_caller_span!())
                    .map_err(|_| RecvError::Disconnected);
                SelectOutcome::Recv {
                    index: ReceiverIndex(user_pos),
                    result,
                }
            }
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
    (Sender { inner: tx }, Receiver { inner: ReceiverKind::Channel(rx) })
}

/// Construct a one-shot timer `Receiver<T>` (arc 292 `:wat::kernel::after`).
///
/// The returned receiver fires exactly once after `duration`, delivering
/// `msg`. After that, subsequent `recv()` or `select()` calls on this
/// receiver return `Err(Disconnected)`.
///
/// Internally uses `crossbeam_channel::after(duration)` — a futex-based
/// wait, NOT `thread::sleep`. No background thread is spawned.
pub fn timer<T: Send + 'static>(duration: std::time::Duration, msg: T) -> Receiver<T> {
    Receiver {
        inner: ReceiverKind::Timer {
            instant_rx: crossbeam_channel::after(duration),
            msg: std::sync::Arc::new(crate::rust_deps::custodia::OwnedMoveCell::new(msg)),
        },
    }
}

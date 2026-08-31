//! vigilatum: 2026-06-01T04:27:02Z — vigilia 9-spell L1+L2=0 (clippy-zero: 5 excusare(perennial) allows)
//!
//! # Comms layer — substrate-internal tier primitives
//!
//! Layer 0a of arc 214's concurrency toolkit (the comms-layer redesign
//! that unifies thread + process tier surfaces under shared traits; full
//! design rationale at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! This module defines the tier-agnostic abstractions (EdnRepresentable
//! wire form, CommSender / CommReceiver traits, error types, SelectOutcome)
//! shared by the thread tier (`comms::thread`) and process tier
//! (`comms::process`) implementations.
//!
//! ## Stone 214 1b-ii-ε — try_recv annihilated
//!
//! `try_recv` (non-blocking recv) has been removed from the substrate.
//! The last call was a `SHUTDOWN_RX` peek (try_recv) in `channel/transfer.rs`
//! used to distinguish `RecvError::Shutdown` from `RecvError::Disconnected`
//! after a blocking recv returned `Err`. This peek has been eliminated by
//! carrying the cause in `RecvError` as a two-variant enum — the comms select
//! already knows which arm fired. `TryRecvError` is removed.
//!
//! ## Cascade contract (LOAD-BEARING)
//!
//! Every blocking method on tier-specific Receivers + Selects MUST wake on
//! substrate shutdown:
//!
//! - Thread tier: `crossbeam_channel::select! { recv(data), recv(SHUTDOWN_RX) }`
//!   — substrate's shutdown cascade signals via crossbeam channel; tier recv
//!   includes this in its select arm. `SHUTDOWN_RX:
//!   OnceLock<crossbeam_channel::Receiver<()>>` lives at `crate::runtime`;
//!   initialized by `init_shutdown_signal()` at `freeze.rs:233` before any
//!   wat code executes. Pre-init the recv falls back to bare crossbeam recv.
//! - Process tier: `io_uring` multi-arm submission on [data_fd, broadcast_fd]
//!   — substrate's broadcast pipe acts as the wake signal; first completion
//!   wins. `SHUTDOWN_BROADCAST_READ_FD: AtomicI32` lives at `crate::runtime`
//!   (init at `freeze.rs:233`); the worker holds the write-end and drops it
//!   on `trigger_shutdown()`, sending POLLHUP to all read-side receivers.
//!   Pre-init (`fd == -1`) the recv falls back to bare io_uring Read.
//!
//! **Intended invariant (NOT YET ENFORCED — Slice 6, pending):** tier wrappers
//! should be the only path to the underlying mechanism. Today bare
//! `crossbeam_channel::*` and bare `libc::pipe/read/write/poll/epoll/io_uring_*`
//! remain reachable elsewhere in the crate; the structural wall (`pub(crate)`
//! visibility reorg) lands in arc-214 Slice 6 (see
//! `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md` § Slice 6).
//!
//! ## Mini-TCP at depth 1 (universal discipline)
//!
//! Each tier vends EXACTLY ONE factory: `pair()`. Both tiers enforce the
//! mini-TCP usage discipline (per `docs/ZERO-MUTEX.md` § "Mini-TCP via
//! paired channels" line 252+): each send pairs with a recv before the
//! next send. The backpressure mechanisms differ by tier:
//!
//! - **Thread tier:** crossbeam `bounded(1)` — structurally capacity-1;
//!   `send` blocks when one value is queued; `recv` drains it.
//! - **Process tier:** Linux anonymous pipe — kernel-buffered (~65536
//!   bytes); individual writes ≤ `PIPE_BUF` (4096 bytes) are POSIX-atomic,
//!   but the pipe is NOT capacity-1. Many frames can accumulate before the
//!   writer blocks. The mini-TCP *discipline* (one send ↔ one recv) is a
//!   usage convention the process tier does not structurally enforce at
//!   depth-1 (per DESIGN.md § "Slice 2 forward-correction (2026-05-19)
//!   — Mini-TCP at depth 1 (universal symmetry)").
//!
//! The substrate doesn't enforce the pairing site-by-site, but both tiers
//! provide backpressure — the thread tier via a hard capacity ceiling, the
//! process tier via the kernel pipe buffer filling under sustained load.
//!
//! There is no `bounded(N)` factory at any tier. The trading-lab
//! convergence (pre-wat-rs origin) proved N > 1 produces massive perf
//! hits + entire categories of problems. See `docs/arc/2026/05/
//! 214-concurrency-toolkit/DESIGN.md` § "Slice 2 forward-correction
//! (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)" for the
//! four-questions verdict + universal symmetry table. Tier-specific
//! detail in `crate::comms::thread` and `crate::comms::process` module
//! docs.
//!
//! ## Audience
//!
//! - **Substrate authors** (building brackets, services, kernel-layer dispatch)
//!   use this module directly via `crate::comms::thread::*` / `crate::comms::
//!   process::*`.
//! - **User code** does NOT touch this layer; uses peer-oriented `:wat::kernel::*`
//!   verbs (Slice 4) that internally dispatch to the right tier.

// ─── Wire form traits ───────────────────────────────────────────────────────

/// Plain-EDN wire contract — Stone C0b.2e-i-0 (arc 209).
///
/// Any type that can be transmitted across a process or remote tier boundary
/// must implement this trait. The contract is **plain EDN**: `to_wire` produces
/// a newline-free EDN string; `from_wire` decodes it back. No HolonAST IR is
/// required — this is the honest minimum the comms wire needs, and (Stone
/// 294.h, see `docs/arc/2026/06/294-holon-returns-to-vsa/`) the ONLY wire
/// trait. A former holographic supertrait — carrying the HolonAST IR
/// conversion methods — had zero production consumers: every process-tier
/// bound was already this trait, and `String` / `Value`, the entire
/// production wire set, were always plain EDN. That supertrait has been
/// deleted; this trait is the wire contract, full stop.
///
/// Thread-tier (in-process) channels also accept `EdnRepresentable` types,
/// but pass `T` directly via crossbeam (no serialization roundtrip).
pub trait EdnRepresentable: Send + 'static {
    /// Encode `self` as a newline-free EDN string for transmission.
    fn to_wire(&self) -> String;
    /// Decode an EDN string back to `Self`.
    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized;
}

/// `EdnRepresentable` for `String` — Stone C0b.2e-i-0.
///
/// Raw passthrough (Stone 214 1b-ii-β.0): the `String` IS the EDN line. No
/// holon tag — a forms-server's `(println 42)` writes plain `42\n`, and the
/// parent reads it back byte-for-byte. The boundary codec (`value_to_edn` /
/// `edn_string_to_value` at the `send'`/`recv'` intrinsics) already turned the
/// Value into this EDN line, so the channel must not re-encode it.
impl EdnRepresentable for String {
    fn to_wire(&self) -> String {
        self.clone()
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        Ok(s.to_string())
    }
}

// ─── EdnRepresentable for Value ─────────────────────────────────────────────

/// `EdnRepresentable` for `Value` — Stone C0b.2e-i-0 (arc 209).
///
/// Plain-EDN wire — no holon tags. `to_wire` uses `value_to_edn_string`
/// (registry-free codec; positional `:field-{i}` for structs). This is used by
/// thread-tier `CommSender<Value>` (crossbeam, no serialisation roundtrip) and
/// as a fallback.
///
/// Arc 258.5b-ii: the socket-tier PEER_TYPE_PATH send path now uses
/// `Peer::send_wire(String)` with the string pre-encoded by
/// `value_to_edn_string_with(v, sym.types())` in the eval layer — `to_wire` is
/// NOT called on that path.  `to_wire` remains for the thread-tier `CommSender`
/// contract and any non-PEER_TYPE_PATH comms (process-tier thread-local is gone).
///
/// `from_wire` uses `edn_string_to_value` with `None` for the type registry
/// (primitive scaffold only — reconstructs i64/f64/bool/nil/String/keyword/
/// Vec/HashMap; user-defined structs are not reconstructed without a TypeEnv).
///
/// `Value` is a plain wat value that serializes as plain EDN, not a
/// holographic value with a HolonAST IR.
///
/// STOP-2 check passed: `edn_string_to_value` passes `None` for the type
/// registry internally (`read_edn(s, None)`) — no SymbolTable / TypeEnv
/// needed at the comms layer.
impl EdnRepresentable for crate::value::Value {
    fn to_wire(&self) -> String {
        // `None` is FORCED here, not chosen: `EdnRepresentable::to_wire(&self)` takes no
        // registry by signature, so a user record crossing THIS path renders positionally.
        // 258.5b-ii already moved the socket tier OFF it (encode in the eval layer, ship
        // bytes) precisely for this reason; the thread tier passes `T` directly. Anything
        // still reaching here with a record is a plumbing gap, not a rendering choice.
        crate::edn::render::value_to_edn_string_with(self, None)
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        // Arc 272 6a-i — `from_wire` is the GENERAL `Value` deserializer; it does NOT assume a
        // trusted channel, so it REFUSES capability tags. The trusted peer wire is
        // the `recv'`/`select'` eval path (runtime.rs), which calls `decode_trusted_wire` directly —
        // the one audited door that may reconstruct a capability (ocap transfer-only).
        crate::edn::render::edn_string_to_value(s)
            .map_err(|e| WireError::new(format!("Value from_wire: {e}")))
    }
}

// ─── Tier-agnostic sender / receiver traits ─────────────────────────────────

/// Which wait-primitive demuxes a receiver in `select'` — Stone C0b.2e-i-a.
/// `InMemory` = parked-thread crossbeam-select (no fd). `Fd` = kernel fd-poll
/// (io_uring). A closed enum on a fixed axis (two wait primitives; a third OS
/// poller is still `Fd`); the growing remote-transport axis lives in the impls,
/// every one fd-backed → `Fd`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReactorClass {
    InMemory,
    Fd,
}

/// Tier-agnostic send endpoint. Implemented by `comms::thread::Sender<T>` (Slice 2)
/// and `comms::process::Sender<T>` (Slice 3). Enables tier-agnostic generic
/// functions for brackets + services that work across both transport layers.
pub trait CommSender<T> {
    fn send(&self, value: T) -> Result<(), SendError<T>>;
    /// Best-effort, GENUINELY non-blocking send. Returns `Err(SendError(value))`
    /// immediately — never blocks — when the channel is full (thread tier's
    /// bounded(1) slot already occupied) or the far side is gone, instead of
    /// waiting for capacity like [`Self::send`] does.
    ///
    /// Arc 278 RST stone (`docs/arc/2026/06/278-rules-engine/
    /// DESIGN-STONE-rst-peer-notify.md`): the ONLY sender used by the
    /// best-effort `PeerCrashed` broadcast (`kernel::peer::Peer::
    /// notify_peer_crashed_best_effort`) — a dying process cannot wait for a
    /// peer to drain its channel. NOT a general-purpose replacement for
    /// `send`; ordinary reply/request traffic keeps the mini-TCP blocking
    /// discipline documented at this module's top.
    ///
    /// Arc 278 Phase 3a (send'-outcome wall, `try-send'`'s own
    /// `TrySendOutcome`): returns [`TrySendError`], not the bare `SendError`
    /// — distinguishes "the channel is full / peer not draining" (a LIVE
    /// peer, `TrySendError::Full`) from "the peer is gone"
    /// (`TrySendError::Disconnected`), instead of collapsing both into one
    /// failure shape. The thread tier mirrors crossbeam's own
    /// `TrySendError::{Full,Disconnected}` split exactly; the process
    /// (pipe) tier derives the same split from the write's errno
    /// (EAGAIN/EWOULDBLOCK ⇒ `Full`, anything else ⇒ `Disconnected`).
    fn try_send(&self, value: T) -> Result<(), TrySendError<T>>;
    /// Signal end-of-stream from this sender. Consumes self so the endpoint
    /// is gone after close. Other cloned `Sender` handles (if any) remain
    /// valid. Peer receivers will see `RecvError::Disconnected`
    /// on their next recv only after ALL `Sender` clones close.
    ///
    /// Infallible: consuming `self` IS the close (Drop handles OS cleanup).
    /// The type system enforces single-close via move semantics — calling
    /// `close` twice is a compile error, not a runtime error.
    fn close(self);
}

/// Tier-agnostic receive endpoint. Implemented by `comms::thread::Receiver<T>` (Slice 2)
/// and `comms::process::Receiver<T>` (Slice 3). Enables tier-agnostic generic
/// functions for brackets + services that work across both transport layers.
///
/// Every blocking method on tier-specific implementations MUST wake on
/// substrate shutdown (cascade contract documented in this module's top-level doc).
// rune:excusare(perennial) — is_empty() structurally withheld: the process tier's len() is a kernel-invisible approximation (kernel-pipe bytes not-yet-drained are invisible); self.len()==0 returns true while unread frames sit in the pipe, so a naive is_empty() would mislead. The transport-oblivion model makes this asymmetry permanent; any change to the process pipe transport would trip the comms ward first. (Documented narrowed-len contract; 9-spell cast.)
#[allow(clippy::len_without_is_empty)]
pub trait CommReceiver<T>: std::any::Any {
    /// Cascade-aware blocking recv. Wakes on substrate shutdown (returns
    /// `Err(RecvError)` when all senders are dropped or the substrate signals
    /// shutdown). Tier implementations wire the shutdown signal automatically —
    /// callers cannot bypass the cascade.
    fn recv(&self) -> Result<T, RecvError>;
    /// Number of values locally buffered and ready for immediate `recv`.
    ///
    /// Non-blocking; cascade-irrelevant.
    ///
    /// **Contract:** implementations MAY undercount when transport-buffered
    /// values are not yet locally drained. The process tier counts only
    /// frames already in the receiver's in-process accumulator; kernel-pipe
    /// bytes not yet read are invisible until consumed via `recv`.
    /// The thread tier is exact (crossbeam's bounded(1) length).
    ///
    /// Useful for capacity-tracking callers (e.g.,
    /// `wat::kernel::HandlePool` checking for orphaned handles).
    fn len(&self) -> usize;
    /// Signal end-of-stream from this receiver. Consumes self so the endpoint
    /// is gone after close. Other cloned `Receiver` handles (if any) remain
    /// valid. Peer senders will see `SendError` on their next `send` only after
    /// ALL `Receiver` clones close.
    ///
    /// Infallible: consuming `self` IS the close (Drop handles OS cleanup).
    /// The type system enforces single-close via move semantics — calling
    /// `close` twice is a compile error, not a runtime error.
    fn close(self);
    /// The wait-primitive class `select'` groups this receiver under.
    fn reactor_class(&self) -> ReactorClass;
    /// Recover the concrete receiver (the i-b `select'` reactor bridge).
    fn as_any(&self) -> &dyn std::any::Any;
}


// ─── Error types ─────────────────────────────────────────────────────────────

/// Send failed — the send-side twin of [`RecvError`] (arc 278
/// send-mirrors-recv, `DESIGN-STONE-send-mirrors-recv.md`). Every arm carries
/// the unsent `T` so the existing recover-or-resend contract survives.
///
/// Built by enumerating `RecvError`'s variants and demanding a send-side
/// meaning for each: two were holes (send was constructed recv-first, and
/// nobody read a send failure to notice) and one is a deliberate non-mirror:
///
/// - [`SendError::Disconnected`] mirrors `RecvError::Disconnected` — EPIPE,
///   the reader's end is gone.
/// - [`SendError::Shutdown`] mirrors `RecvError::Shutdown` — the substrate
///   shutdown broadcast fired mid-write (the write was polled, not blindly
///   blocked — see `comms::process::Sender::send`).
/// - [`SendError::Failed`] mirrors `RecvError::Failed` — a raw io error,
///   carrying its reason, per the arc 278 no-hidden-failures law.
/// - `RecvError::PeerCrashed` does NOT mirror: on recv it comes off the
///   crash channel, but a sender has no crash-channel access at this layer
///   and sees a dead peer as EPIPE, which honestly collapses into
///   `Disconnected`. Recorded here rather than silently omitted — silence is
///   exactly what produced the other holes.
/// - `RecvError::FrameTooLarge` does NOT mirror: the transport cannot know
///   which *op* is being sent, so it can never hold the right per-op budget
///   (arc 278 "cut the cap, prove the poll arm" — `BRIEF-prove-the-poll-arm.md`).
///   The check moves to the generated client method in a later strike; the
///   receiver's own `FrameTooLarge` dismissal stays as the sole backstop.
#[derive(Debug)]
pub enum SendError<T> {
    /// A genuine clean close: the peer's read end is gone (EPIPE on the
    /// process tier; every `Receiver` dropped on the thread tier).
    Disconnected(T),
    /// The substrate shutdown cascade fired while this send was blocked
    /// waiting for room (the broadcast / `SHUTDOWN_BROADCAST_READ_FD` arm).
    /// The peer is not known to be gone — only that the write was told to
    /// stop waiting.
    Shutdown(T),
    /// A raw transport failure with a carried reason: an io error other than
    /// EPIPE (e.g. `EIO`). The `String` is the underlying error's
    /// `to_string()`, per the arc 278 no-hidden-failures law — the
    /// send-tier twin of `RecvError::Failed`.
    Failed(T, String),
}

/// Genuinely non-blocking send failed — Arc 278 Phase 3a. Distinguishes WHY
/// a `try_send` did not land — unlike the blocking [`SendError`], which now
/// distinguishes disconnect / shutdown / an over-cap frame / a raw io error
/// but still has no "full" case (a blocking send never returns "full", it
/// just waits — that one axis of the old "the distinction is moot" rationale
/// was correct; the other three were not, which is why `SendError` is now an
/// enum). Shape mirrors `crossbeam_channel::TrySendError<T>` exactly for the
/// thread tier; the process (pipe) tier derives the same two arms from the
/// write's errno.
#[derive(Debug)]
pub enum TrySendError<T> {
    /// The channel's bounded slot is occupied (thread tier) / the pipe
    /// buffer is full (process tier, EAGAIN/EWOULDBLOCK) — a LIVE peer just
    /// not draining fast enough. Retry-able in principle; `try_send`
    /// callers treat it as a best-effort skip, never a retry loop.
    Full(T),
    /// The receiver is gone (thread tier: all `Receiver`s dropped; process
    /// tier: any write failure other than EAGAIN/EWOULDBLOCK, e.g. EPIPE).
    Disconnected(T),
}

/// Recv failed — carrying the cause the comms select already computes
/// (Stone 214 1b-ii-ε). The select fires on a specific arm and *knows* whether
/// it was a data disconnect or a substrate shutdown. Carrying the distinction
/// in this enum lets consumers match the variant directly without a secondary
/// `SHUTDOWN_RX` peek. `try_recv` has been annihilated from the substrate.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RecvError {
    /// A genuine CLEAN close ONLY: all senders dropped / the peer closed the
    /// write-end with no error (EOF / data arm). NEVER produced for a raw
    /// transport failure (io error, invalid UTF-8, undecodable/malformed
    /// frame) — those carry a reason via [`RecvError::Failed`] instead, per
    /// the arc 278 no-hidden-failures law (the transport-tier twin of the
    /// service-reply / crash-reason mechanisms). Mute-collapsing a real
    /// error into `Disconnected` is exactly the mislabeling this variant's
    /// contract forbids.
    Disconnected,
    /// The substrate shutdown cascade fired (the broadcast / `SHUTDOWN_RX` arm).
    Shutdown,
    /// The accumulator exceeded `DEFAULT_MAX_FRAME_BYTES` without a complete
    /// frame (no newline terminator). The peer wrote more than the cap without
    /// ending a frame — its message is rejected. Distinct from `Disconnected`
    /// so callers can tear down the peer WITHOUT reading the error channel
    /// (the peer is still alive; blocking on err.recv() would deadlock).
    /// Stays its own variant — NEVER folded into `Failed` (callers must not
    /// read the err channel for it; see the doc above).
    FrameTooLarge,
    /// The far side was SEVERED: a service's owner released its handle, the
    /// lineage channel drained, and the serve loop exited. Delivered as the
    /// reserved `PEER_SEVERED_SENTINEL` on the peer's existing data channel
    /// (`kernel::peer`), exactly as `PeerCrashed` is.
    ///
    /// Its own variant, never folded into `Disconnected` (which would restore
    /// the mute: a clean-close label on a service that did not close cleanly)
    /// and never into `PeerCrashed` (nothing crashed — mislabeling an orderly
    /// owner-drop as an abnormal death is the class of lie arc 170 pulled out
    /// when a healthy stopped peer was reported as "peer closed").
    ///
    /// BEST-EFFORT, like `PeerCrashed`: the sentinel is `try_send`, so a torn-down
    /// pipe can beat it and the client then reads `Disconnected`. Its ARRIVAL is
    /// information; its ABSENCE proves nothing about the owner.
    PeerSevered,
    /// A raw transport failure with a carried reason: an io_uring
    /// submission/read error, invalid UTF-8 in a frame, a wire (EDN)
    /// decode failure, or a frame-scan malformed-frame rejection. The
    /// `String` is the underlying error's `to_string()` (or an equivalent
    /// diagnostic) so a caller can tell a genuine wire break apart from a
    /// clean peer close instead of both collapsing to a mute
    /// `Disconnected`. Arc 278 no-hidden-failures — the transport-tier twin.
    Failed(String),
    /// The far side crashed abnormally (an unhandled panic mid-handler),
    /// NOT a clean close. Distinct from `Disconnected` (a genuine clean
    /// FIN — the far side exited with no error) — `PeerCrashed` is the
    /// RST-in-nature signal: a best-effort, reason-free notification a
    /// dying `defservice` serve loop sends to every connected peer before
    /// it lets itself crash (arc 278 tail — see
    /// `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-rst-peer-notify.md`).
    /// Carries NO reason: the crash reason is administrative (arc 294 —
    /// "a crash reason is administrative, to the creator, never blind
    /// callers") and travels ONLY to the owner's crash channel
    /// (`PeerRecvError::Crashed`), never to a `connect'`-ed peer. Recognized
    /// as a reserved sentinel on the peer's existing data channel
    /// (`kernel::peer::Peer::recv`/`recv_wire`) — there is no separate
    /// control channel at either transport tier.
    PeerCrashed,
}

/// Wire roundtrip failure during EDN serialization/deserialization.
///
/// Produced by an `EdnRepresentable::from_wire` implementation when the
/// incoming EDN does not decode to a valid payload.
///
/// Field is private so only `EdnRepresentable` impls construct via `new()`;
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

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WireError {}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvError::Disconnected => f.write_str("channel disconnected"),
            RecvError::Shutdown => f.write_str("substrate shutdown"),
            RecvError::FrameTooLarge => f.write_str("frame exceeded cap (message larger than the receiver's max-message-bytes budget)"),
            RecvError::Failed(reason) => write!(f, "transport failed: {reason}"),
            RecvError::PeerCrashed => f.write_str("peer crashed (abnormal far-side crash — no reason; the crash reason is administrative and travels only to the owner's crash channel)"),
            RecvError::PeerSevered => f.write_str("service severed: its owner released the service handle, so the serve loop exited (the lineage peer drained)"),
        }
    }
}

impl std::error::Error for RecvError {}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::Disconnected(_) => f.write_str("send failed: channel disconnected"),
            SendError::Shutdown(_) => f.write_str("send failed: substrate shutdown"),
            SendError::Failed(_, reason) => write!(f, "send failed: transport failed: {reason}"),
        }
    }
}

impl<T: std::fmt::Debug> std::error::Error for SendError<T> {}

// ─── Tier modules ────────────────────────────────────────────────────────────

/// Thread tier: in-process comms via crossbeam_channel. Cascade-aware.
/// Substrate-internal; user code uses `:wat::kernel::*` verbs (Slice 4).
pub mod thread;

/// Process tier: cross-process comms via io_uring + anonymous pipes.
/// Cascade-aware (Stone B). Substrate-internal; user code uses
/// `:wat::kernel::*` verbs (Slice 4).
pub mod process;

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
///
/// io_uring substrate failures (`process::Select`) are NOT represented
/// here — they surface as `Err(std::io::Error)` on `process::Select::select()`
/// (which returns `Result<SelectOutcome<T>, std::io::Error>`). The thread
/// tier's `Select::select()` returns bare `SelectOutcome<T>` (the thread
/// tier has no io_uring failure mode and is infallible beyond Recv/Shutdown).
/// This asymmetry is HONEST: the tiers genuinely differ; a shared
/// `SubstrateError` variant would force the thread tier to handle an
/// impossible arm.
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
    /// Arc 209 C0b.3a-i — the registered listener arm fired (a connection is pending).
    /// The caller accepts (non-blocking) and wraps the new connection.
    Listener,
}

#[cfg(test)]
mod beta0_wire_tests {
    //! Stone 214 1b-ii-β.0 — the universe-boundary wire is plain EDN, never a
    //! holon-tagged envelope. Holon-tagging is one representation of EDN, content
    //! INSIDE a holonic value — not the transport.
    use super::EdnRepresentable;

    #[test]
    fn string_wire_is_raw_edn_not_holon_tagged() {
        // The process peer's String IS the finished EDN line (the send'/recv'
        // boundary codec ran value_to_edn upstream). to_wire must NOT re-wrap it.
        let edn_line = "42".to_string();
        assert_eq!(
            edn_line.to_wire(),
            "42",
            "String::to_wire must be raw passthrough — a forms-server's plain `42` is the wire"
        );
        assert_eq!(edn_line.to_wire(), "42", "the wire must carry no holon-AST envelope");
        assert_eq!(String::from_wire("42").unwrap(), "42");

        // A tagged literal (#wat.kernel.LociDiedError/...) is itself valid EDN and
        // rides the wire as-is — proving holon tags are content, not envelope.
        // (arc 278: the bare `Vector<LociDiedError>` death chain is exactly such a
        // self-describing tagged line; the old `#wat.kernel/ProcessPanics` wrapper
        // was annihilated.)
        let tagged = "#wat.kernel.LociDiedError/Stopped []".to_string();
        assert_eq!(tagged.to_wire(), "#wat.kernel.LociDiedError/Stopped []");
        assert_eq!(
            String::from_wire("#wat.kernel.LociDiedError/Stopped []").unwrap(),
            "#wat.kernel.LociDiedError/Stopped []"
        );
    }
}

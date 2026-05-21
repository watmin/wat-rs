//! # Comms layer — substrate-internal tier primitives
//!
//! Layer 0a of arc 214's concurrency toolkit (the comms-layer redesign
//! that unifies thread + process tier surfaces under shared traits; full
//! design rationale at `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`).
//! This module defines the tier-agnostic abstractions (HolonRepresentable
//! wire form, CommSender / CommReceiver traits, error types, SelectOutcome)
//! shared by the thread tier (`comms::thread`) and process tier
//! (`comms::process`) implementations.
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
//! Callers cannot bypass the cascade because tier wrappers hide the underlying
//! mechanism. Bare `crossbeam_channel::*` and bare `libc::pipe/read/write/
//! poll/epoll/io_uring_*` are unreachable outside the tier wrapper modules
//! (Slice 6 structural wall).
//!
//! ## Mini-TCP at depth 1 (universal discipline)
//!
//! Each tier vends EXACTLY ONE factory: `pair()`. The factory returns a
//! capacity-1 channel pair (thread: crossbeam `bounded(1)`; process: OS
//! pipe, kernel-bounded by `PIPE_BUF`). `send` blocks when the buffer
//! holds one value; `recv` drains it.
//!
//! Capacity-1 is the structural enforcement of the mini-TCP usage
//! discipline (per `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels"
//! line 252+): each send pairs with a recv before the next send. The
//! substrate doesn't enforce the pairing site-by-site, but capacity-1
//! makes producers that try to outpace consumers block immediately
//! rather than queuing up and amplifying drift.
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

/// First concrete `HolonRepresentable` impl (Slice 3 Stone C).
///
/// Encodes `String` as `HolonAST::String`. The roundtrip is exact —
/// `String::from_holon_ast(s.to_holon_ast())` returns the original
/// string (including any embedded `'\n'` which wat-edn escapes
/// during serialization).
///
/// Used by Stone C's probe tests as the test type. Future arcs may
/// add impls for other substrate types (StdInServiceEvent,
/// SpawnOutcome, etc.) as Slice 4/5 consumers require.
impl HolonRepresentable for String {
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::String(self.as_str().into())
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::String(s) => Ok(s.to_string()),
            other => Err(WireError::new(format!(
                "expected HolonAST::String, got {:?}",
                other
            ))),
        }
    }
}

/// Arc 216 Stone 1 — `HolonRepresentable` for `HashSet<T>`.
///
/// Mirrors the `String` impl pattern (line 107). Encodes as
/// `HolonAST::Bundle(vec![T_holon, T_holon, ...])` — the set-shape per
/// DESIGN Q2/Q3 (bare atoms, no Bind keys). Dedupe is enforced at
/// construction time (HashSet insert is idempotent); the Bundle carries
/// one child per unique element.
///
/// `from_holon_ast` reconstructs the HashSet by matching `Bundle` shape,
/// converting each child via `T::from_holon_ast`, and inserting into the set.
/// Duplicate atoms (if any were in the Bundle) dedup naturally.
///
/// Bounds: `T: HolonRepresentable + std::hash::Hash + Eq + Send + 'static`
/// mirrors the BRIEF's `impl<T> HolonRepresentable for HashSet<T>` shape.
/// `Hash + Eq` are required for the inner `HashSet<T>` type; `Send + 'static`
/// are required by the `HolonRepresentable` supertrait.
impl<T> HolonRepresentable for std::collections::HashSet<T>
where
    T: HolonRepresentable + std::hash::Hash + Eq + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        let children: Vec<holon::HolonAST> = self.iter().map(|v| v.to_holon_ast()).collect();
        holon::HolonAST::bundle(children)
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::Bundle(items) => {
                let mut set = std::collections::HashSet::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    let v = T::from_holon_ast(item).map_err(|e| {
                        WireError::new(format!(
                            "HashSet element #{} failed HolonRepresentable::from_holon_ast: {}",
                            i, e.message()
                        ))
                    })?;
                    set.insert(v);
                }
                Ok(set)
            }
            other => Err(WireError::new(format!(
                "expected HolonAST::Bundle (set-shape), got {:?}",
                other
            ))),
        }
    }
}

/// Arc 216 Stone 2 — `HolonRepresentable` for `Vec<T>`.
///
/// Mirrors the `HashSet<T>` impl pattern (Stone 1). Encodes as
/// `HolonAST::Bundle(vec![Bind(I64(0), T_holon), Bind(I64(1), T_holon), ...])` —
/// the array-shape per DESIGN Q2 (positional-Bind keys 0..n-1). Order is
/// preserved — index 0 maps to Bind key 0, preserving element sequence.
///
/// `from_holon_ast` reconstructs the Vec by matching `Bundle` shape,
/// verifying all children are `Bind(I64(_), _)` with sequential keys 0..n-1,
/// and converting each Bind's value via `T::from_holon_ast` in key order.
///
/// Bounds: `T: HolonRepresentable + Send + 'static` — no `Hash + Eq`
/// required (unlike HashSet) because Vec elements need not be hashable.
impl<T> HolonRepresentable for Vec<T>
where
    T: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        let children: Vec<holon::HolonAST> = self
            .iter()
            .enumerate()
            .map(|(i, v)| holon::HolonAST::bind(holon::HolonAST::i64(i as i64), v.to_holon_ast()))
            .collect();
        holon::HolonAST::bundle(children)
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::Bundle(items) => {
                // Validate all children are Bind(I64(_), _).
                for (pos, item) in items.iter().enumerate() {
                    match item {
                        holon::HolonAST::Bind(k, _) => {
                            if !matches!(k.as_ref(), holon::HolonAST::I64(_)) {
                                return Err(WireError::new(format!(
                                    "Vec element #{} Bind key is not I64 (expected positional integer key)",
                                    pos
                                )));
                            }
                        }
                        other => {
                            return Err(WireError::new(format!(
                                "Vec element #{} is not a Bind (expected positional-Bind vector-shape); got {:?}",
                                pos, other
                            )));
                        }
                    }
                }
                // Collect (key, value_ast) pairs, sort by key, validate 0..n-1 sequential.
                let n = items.len();
                let mut pairs: Vec<(i64, &holon::HolonAST)> = Vec::with_capacity(n);
                for item in items.iter() {
                    match item {
                        holon::HolonAST::Bind(k, v) => {
                            let idx = match k.as_ref() {
                                holon::HolonAST::I64(i) => *i,
                                _ => unreachable!("already validated"),
                            };
                            pairs.push((idx, v.as_ref()));
                        }
                        _ => unreachable!("already validated"),
                    }
                }
                pairs.sort_by_key(|(k, _)| *k);
                for (expected, (actual, _)) in pairs.iter().enumerate() {
                    if *actual != expected as i64 {
                        return Err(WireError::new(format!(
                            "Vec positional invariant violated: expected key {} at position {}, got {}",
                            expected, expected, actual
                        )));
                    }
                }
                // Reconstruct in order.
                let mut out: Vec<T> = Vec::with_capacity(n);
                for (pos, (_, v_ast)) in pairs.iter().enumerate() {
                    let v = T::from_holon_ast(v_ast).map_err(|e| {
                        WireError::new(format!(
                            "Vec element #{} failed HolonRepresentable::from_holon_ast: {}",
                            pos,
                            e.message()
                        ))
                    })?;
                    out.push(v);
                }
                Ok(out)
            }
            other => Err(WireError::new(format!(
                "expected HolonAST::Bundle (vector-shape), got {:?}",
                other
            ))),
        }
    }
}

/// Arc 216 Stone 3 — `HolonRepresentable` for `HashMap<K, V>`.
///
/// Mirrors the `HashSet<T>` (Stone 1) and `Vec<T>` (Stone 2) patterns.
/// Encodes as a `HolonAST::Bundle` of `Bind(K_holon, V_holon)` pairs
/// (arbitrary-K map-shape per DESIGN Q2). Iteration order is non-canonical
/// (HashMap unordered); the Bundle's Bind order is therefore non-deterministic.
/// Round-trip is correct because the reverse trip reconstructs a HashMap which
/// is also order-agnostic.
///
/// `from_holon_ast` validates the Bundle-of-Bind shape; each Bind's key and
/// value are decoded via `K::from_holon_ast` and `V::from_holon_ast`
/// respectively. The canonical key for the output HashMap is computed via
/// `K::to_holon_ast` → `hashmap_key_from_holon` (deterministic string key).
///
/// Bounds: `K: HolonRepresentable + std::hash::Hash + Eq + Send + 'static` —
/// Hash + Eq required for the inner `HashMap<K, V>` type; mirrors `HashSet<T>`
/// bounds. `V: HolonRepresentable + Send + 'static` — no Hash + Eq needed.
impl<K, V> HolonRepresentable for std::collections::HashMap<K, V>
where
    K: HolonRepresentable + std::hash::Hash + Eq + Send + 'static,
    V: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        let items: Vec<holon::HolonAST> = self
            .iter()
            .map(|(k, v)| {
                let k_holon = k.to_holon_ast();
                let v_holon = v.to_holon_ast();
                holon::HolonAST::bind(k_holon, v_holon)
            })
            .collect();
        holon::HolonAST::bundle(items)
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        match ast {
            holon::HolonAST::Bundle(items) => {
                let mut map = std::collections::HashMap::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    match item {
                        holon::HolonAST::Bind(k_holon, v_holon) => {
                            let k = K::from_holon_ast(k_holon).map_err(|e| {
                                WireError::new(format!(
                                    "HashMap key #{} failed HolonRepresentable::from_holon_ast: {}",
                                    i,
                                    e.message()
                                ))
                            })?;
                            let v = V::from_holon_ast(v_holon).map_err(|e| {
                                WireError::new(format!(
                                    "HashMap value #{} failed HolonRepresentable::from_holon_ast: {}",
                                    i,
                                    e.message()
                                ))
                            })?;
                            map.insert(k, v);
                        }
                        other => {
                            return Err(WireError::new(format!(
                                "expected HolonAST::Bind at index {}; got {:?}",
                                i, other
                            )));
                        }
                    }
                }
                Ok(map)
            }
            other => Err(WireError::new(format!(
                "expected HolonAST::Bundle (map-shape of Bind children), got {:?}",
                other
            ))),
        }
    }
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
    ///
    /// Infallible: consuming `self` IS the close (Drop handles OS cleanup).
    /// The type system enforces single-close via move semantics — calling
    /// `close` twice is a compile error, not a runtime error.
    fn close(self);
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
    /// Substrate-level failure in the Select machinery itself (e.g., io_uring
    /// ring creation, SQE submission, or submit_and_wait failure). Distinct
    /// from any user-arm firing — `ReceiverIndex` is meaningless when the
    /// substrate itself failed. Callers matching exhaustively will see this
    /// arm and can report the error or treat it as fatal.
    SubstrateError(std::io::Error),
}

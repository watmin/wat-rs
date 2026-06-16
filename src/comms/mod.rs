//! vigilatum: 2026-06-01T04:27:02Z — vigilia 9-spell L1+L2=0 (clippy-zero: 5 excusare(perennial) allows)
//!
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
/// required — this is the honest minimum the comms wire needs.
///
/// `HolonRepresentable` is a strict supertrait (`HolonRepresentable:
/// EdnRepresentable`) for types that additionally carry the holographic IR
/// (`to_holon_ast` / `from_holon_ast`). Every `HolonRepresentable` is
/// automatically `EdnRepresentable`; the converse is NOT true (e.g., `Value`
/// implements `EdnRepresentable` but not `HolonRepresentable`).
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

/// Holographic wire form — the `HolonAST` IR layer on top of `EdnRepresentable`.
///
/// Anything that crosses a process or remote tier boundary AND needs the full
/// holographic encoding IR (`HolonAST`) implements this trait. It is a strict
/// supertrait of `EdnRepresentable`; implementors must provide both the plain-EDN
/// methods (via `EdnRepresentable`) and the holographic IR methods here.
///
/// Per project_holon_universal_ast (the strange loop closing 2026-05-19): HolonAST
/// was minted for VSA encoding (arc 057), became universal AST (arc 143 signature
/// reflection, arc 201 type reflection), and is NOW also the universal comms wire
/// form for holographic types.
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
pub trait HolonRepresentable: EdnRepresentable {
    fn to_holon_ast(&self) -> holon::HolonAST;
    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
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

/// First concrete `HolonRepresentable` impl (Slice 3 Stone C).
///
/// Encodes `String` as `HolonAST::String`. The roundtrip is exact —
/// `String::from_holon_ast(s.to_holon_ast())` returns the original
/// string (including any embedded `'\n'` which wat-edn escapes
/// during serialization).
///
/// Used by Stone C's probe tests as the test type. Further impls land
/// per-type as consumers surface them (the Slice-8 service rebirth
/// retired the old `*ServiceEvent` candidates — services now speak
/// `Value`-shaped Req/Rep records, not Rust enums).
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

/// Arc 216 Stone 1 — `EdnRepresentable` for `HashSet<T>` — Stone C0b.2e-i-0.
///
/// Wire via the holon-tagged EDN of `to_holon_ast` (same behavior as before
/// the trait split; tagged stays tagged). Explicit to satisfy `EdnRepresentable`
/// as a required supertrait of `HolonRepresentable`.
impl<T> EdnRepresentable for std::collections::HashSet<T>
where
    T: HolonRepresentable + std::hash::Hash + Eq + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
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

/// Arc 216 Stone 2 — `EdnRepresentable` for `Vec<T>` — Stone C0b.2e-i-0.
///
/// Wire via the holon-tagged EDN of `to_holon_ast` (same behavior as before
/// the trait split; tagged stays tagged). Explicit to satisfy `EdnRepresentable`
/// as a required supertrait of `HolonRepresentable`.
impl<T> EdnRepresentable for Vec<T>
where
    T: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
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

/// Arc 216 Stone 3 — `EdnRepresentable` for `HashMap<K, V>` — Stone C0b.2e-i-0.
///
/// Wire via the holon-tagged EDN of `to_holon_ast` (same behavior as before
/// the trait split; tagged stays tagged). Explicit to satisfy `EdnRepresentable`
/// as a required supertrait of `HolonRepresentable`.
impl<K, V> EdnRepresentable for std::collections::HashMap<K, V>
where
    K: HolonRepresentable + std::hash::Hash + Eq + Send + 'static,
    V: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
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

// ─── HolonRepresentable for Rust tuples ─────────────────────────────────────
//
// Arc 216 Stone 7 — fixed-arity impls for Rust tuples (T1, T2) through (T1, T2, T3, T4, T5).
//
// Arity ceiling: 5. Rationale: 2-5 covers all practical cases in the substrate
// (pairs dominate; triples + quads for multi-result forms; quintuple is the maximum
// observed in any current wat-rs caller). Arity 6+ would require a macro helper
// (STOP-2 trigger threshold) — surfaced if a future stone needs it. Tuples of arity
// 1 are rare and can be wrapped in a 2-tuple; arity 0 is Value::Unit (not Tuple).
//
// Encoding: positional-Bind Bundle — identical to Vec<T> (collection-category per
// encoding doctrine). Bundle([Bind(I64(0), T1_holon), Bind(I64(1), T2_holon), ...]).
// `from_holon_ast` validates Bundle shape and sequential I64 keys; decodes each
// Bind's value via the corresponding T_i::from_holon_ast. Returns WireError on
// arity mismatch, non-sequential keys, or element decode failure.
//
// Bounds per element: `Ti: HolonRepresentable + Send + 'static`.

/// Arc 216 Stone 7 — `EdnRepresentable` for `(T1, T2)` (2-tuple) — Stone C0b.2e-i-0.
impl<T1, T2> EdnRepresentable for (T1, T2)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }
    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
    }
}

/// Arc 216 Stone 7 — `HolonRepresentable` for `(T1, T2)` (2-tuple).
impl<T1, T2> HolonRepresentable for (T1, T2)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::bundle(vec![
            holon::HolonAST::bind(holon::HolonAST::i64(0), self.0.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(1), self.1.to_holon_ast()),
        ])
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let items = extract_positional_binds(ast, 2, "2-tuple")?;
        let t0 = T1::from_holon_ast(items[0]).map_err(|e| WireError::new(format!("2-tuple element 0: {}", e.message())))?;
        let t1 = T2::from_holon_ast(items[1]).map_err(|e| WireError::new(format!("2-tuple element 1: {}", e.message())))?;
        Ok((t0, t1))
    }
}

/// Arc 216 Stone 7 — `EdnRepresentable` for `(T1, T2, T3)` (3-tuple) — Stone C0b.2e-i-0.
impl<T1, T2, T3> EdnRepresentable for (T1, T2, T3)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }
    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
    }
}

/// Arc 216 Stone 7 — `HolonRepresentable` for `(T1, T2, T3)` (3-tuple).
impl<T1, T2, T3> HolonRepresentable for (T1, T2, T3)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::bundle(vec![
            holon::HolonAST::bind(holon::HolonAST::i64(0), self.0.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(1), self.1.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(2), self.2.to_holon_ast()),
        ])
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let items = extract_positional_binds(ast, 3, "3-tuple")?;
        let t0 = T1::from_holon_ast(items[0]).map_err(|e| WireError::new(format!("3-tuple element 0: {}", e.message())))?;
        let t1 = T2::from_holon_ast(items[1]).map_err(|e| WireError::new(format!("3-tuple element 1: {}", e.message())))?;
        let t2 = T3::from_holon_ast(items[2]).map_err(|e| WireError::new(format!("3-tuple element 2: {}", e.message())))?;
        Ok((t0, t1, t2))
    }
}

/// Arc 216 Stone 7 — `EdnRepresentable` for `(T1, T2, T3, T4)` (4-tuple) — Stone C0b.2e-i-0.
impl<T1, T2, T3, T4> EdnRepresentable for (T1, T2, T3, T4)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
    T4: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }
    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
    }
}

/// Arc 216 Stone 7 — `HolonRepresentable` for `(T1, T2, T3, T4)` (4-tuple).
impl<T1, T2, T3, T4> HolonRepresentable for (T1, T2, T3, T4)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
    T4: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::bundle(vec![
            holon::HolonAST::bind(holon::HolonAST::i64(0), self.0.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(1), self.1.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(2), self.2.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(3), self.3.to_holon_ast()),
        ])
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let items = extract_positional_binds(ast, 4, "4-tuple")?;
        let t0 = T1::from_holon_ast(items[0]).map_err(|e| WireError::new(format!("4-tuple element 0: {}", e.message())))?;
        let t1 = T2::from_holon_ast(items[1]).map_err(|e| WireError::new(format!("4-tuple element 1: {}", e.message())))?;
        let t2 = T3::from_holon_ast(items[2]).map_err(|e| WireError::new(format!("4-tuple element 2: {}", e.message())))?;
        let t3 = T4::from_holon_ast(items[3]).map_err(|e| WireError::new(format!("4-tuple element 3: {}", e.message())))?;
        Ok((t0, t1, t2, t3))
    }
}

/// Arc 216 Stone 7 — `EdnRepresentable` for `(T1, T2, T3, T4, T5)` (5-tuple) — Stone C0b.2e-i-0.
impl<T1, T2, T3, T4, T5> EdnRepresentable for (T1, T2, T3, T4, T5)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
    T4: HolonRepresentable + Send + 'static,
    T5: HolonRepresentable + Send + 'static,
{
    fn to_wire(&self) -> String {
        crate::edn_shim::write_holon_ast_tagged(&self.to_holon_ast())
    }
    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let ast = crate::edn_shim::read_holon_ast_tagged(s)
            .map_err(|e| WireError::new(format!("from_wire: {e}")))?;
        Self::from_holon_ast(&ast)
    }
}

/// Arc 216 Stone 7 — `HolonRepresentable` for `(T1, T2, T3, T4, T5)` (5-tuple).
impl<T1, T2, T3, T4, T5> HolonRepresentable for (T1, T2, T3, T4, T5)
where
    T1: HolonRepresentable + Send + 'static,
    T2: HolonRepresentable + Send + 'static,
    T3: HolonRepresentable + Send + 'static,
    T4: HolonRepresentable + Send + 'static,
    T5: HolonRepresentable + Send + 'static,
{
    fn to_holon_ast(&self) -> holon::HolonAST {
        holon::HolonAST::bundle(vec![
            holon::HolonAST::bind(holon::HolonAST::i64(0), self.0.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(1), self.1.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(2), self.2.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(3), self.3.to_holon_ast()),
            holon::HolonAST::bind(holon::HolonAST::i64(4), self.4.to_holon_ast()),
        ])
    }

    fn from_holon_ast(ast: &holon::HolonAST) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        let items = extract_positional_binds(ast, 5, "5-tuple")?;
        let t0 = T1::from_holon_ast(items[0]).map_err(|e| WireError::new(format!("5-tuple element 0: {}", e.message())))?;
        let t1 = T2::from_holon_ast(items[1]).map_err(|e| WireError::new(format!("5-tuple element 1: {}", e.message())))?;
        let t2 = T3::from_holon_ast(items[2]).map_err(|e| WireError::new(format!("5-tuple element 2: {}", e.message())))?;
        let t3 = T4::from_holon_ast(items[3]).map_err(|e| WireError::new(format!("5-tuple element 3: {}", e.message())))?;
        let t4 = T5::from_holon_ast(items[4]).map_err(|e| WireError::new(format!("5-tuple element 4: {}", e.message())))?;
        Ok((t0, t1, t2, t3, t4))
    }
}

/// Shared helper for tuple `from_holon_ast` impls.
///
/// Validates that `ast` is a `HolonAST::Bundle` with exactly `expected_arity` children,
/// all of which are `Bind(I64(i), _)` with sequential keys 0..expected_arity-1.
/// Returns a `Vec` of references to the Bind values in key order (index 0 first).
///
/// Returns `WireError` on:
/// - Not a Bundle
/// - Wrong child count (arity mismatch)
/// - Non-I64 Bind key
/// - Non-sequential keys (gap or duplicate)
fn extract_positional_binds<'a>(
    ast: &'a holon::HolonAST,
    expected_arity: usize,
    context: &str,
) -> Result<Vec<&'a holon::HolonAST>, WireError> {
    let items = match ast {
        holon::HolonAST::Bundle(items) => items,
        other => {
            return Err(WireError::new(format!(
                "{}: expected HolonAST::Bundle (positional-Bind tuple-shape), got {:?}",
                context, other
            )));
        }
    };
    if items.len() != expected_arity {
        return Err(WireError::new(format!(
            "{}: arity mismatch — expected {} Bind children, got {}",
            context, expected_arity, items.len()
        )));
    }
    // Collect (key, value_ref) pairs.
    let mut pairs: Vec<(i64, &holon::HolonAST)> = Vec::with_capacity(expected_arity);
    for (pos, item) in items.iter().enumerate() {
        match item {
            holon::HolonAST::Bind(k, v) => {
                match k.as_ref() {
                    holon::HolonAST::I64(i) => pairs.push((*i, v.as_ref())),
                    other => {
                        return Err(WireError::new(format!(
                            "{}: element #{} Bind key is not I64 (expected positional integer key); got {:?}",
                            context, pos, other
                        )));
                    }
                }
            }
            other => {
                return Err(WireError::new(format!(
                    "{}: element #{} is not a Bind (expected positional-Bind tuple-shape); got {:?}",
                    context, pos, other
                )));
            }
        }
    }
    // Sort by key, validate sequential 0..expected_arity-1.
    pairs.sort_by_key(|(k, _)| *k);
    for (expected, (actual, _)) in pairs.iter().enumerate() {
        if *actual != expected as i64 {
            return Err(WireError::new(format!(
                "{}: positional invariant violated — expected key {} at position {}, got {}",
                context, expected, expected, actual
            )));
        }
    }
    // Return value refs in key order.
    Ok(pairs.into_iter().map(|(_, v)| v).collect())
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
/// `Value` does NOT impl `HolonRepresentable` — it is a plain wat value that
/// serializes as plain EDN, not a holographic value with a HolonAST IR.
///
/// STOP-2 check passed: `edn_string_to_value` passes `None` for the type
/// registry internally (`read_edn(s, None)`) — no SymbolTable / TypeEnv
/// needed at the comms layer.
impl EdnRepresentable for crate::value::Value {
    fn to_wire(&self) -> String {
        crate::edn_shim::value_to_edn_string(self)
    }

    fn from_wire(s: &str) -> Result<Self, WireError>
    where
        Self: Sized,
    {
        // Arc 272 6a-i — `from_wire` is the GENERAL `Value` deserializer; it does NOT assume a
        // trusted channel, so it REFUSES capability (`wat-edn.cap`) tags. The trusted peer wire is
        // the `recv'`/`select'` eval path (runtime.rs), which calls `decode_trusted_wire` directly —
        // the one audited door that may reconstruct a capability (ocap transfer-only).
        crate::edn_shim::edn_string_to_value(s)
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

/// Send failed: receiver was dropped or substrate shut down.
///
/// Holds the unsent value so the caller can inspect or recover it.
/// Shape matches `crossbeam_channel::SendError<T>` for ergonomic familiarity.
#[derive(Debug)]
pub struct SendError<T>(pub T);

/// Recv failed — carrying the cause the comms select already computes
/// (Stone 214 1b-ii-ε). The select fires on a specific arm and *knows* whether
/// it was a data disconnect or a substrate shutdown. Carrying the distinction
/// in this enum lets consumers match the variant directly without a secondary
/// `SHUTDOWN_RX` peek. `try_recv` has been annihilated from the substrate.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RecvError {
    /// All senders dropped / the peer closed the write-end (EOF / data arm).
    Disconnected,
    /// The substrate shutdown cascade fired (the broadcast / `SHUTDOWN_RX` arm).
    Shutdown,
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

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WireError {}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RecvError::Disconnected => "channel disconnected",
            RecvError::Shutdown => "substrate shutdown",
        })
    }
}

impl std::error::Error for RecvError {}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("send failed: channel disconnected")
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
        assert!(
            !edn_line.to_wire().contains("#wat-edn.holon"),
            "the wire must carry no holon-AST envelope"
        );
        assert_eq!(String::from_wire("42").unwrap(), "42");

        // A tagged literal (#wat.kernel/...) is itself valid EDN and rides the
        // wire as-is — proving holon tags are content, not envelope.
        let tagged = "#wat.kernel/ProcessPanics []".to_string();
        assert_eq!(tagged.to_wire(), "#wat.kernel/ProcessPanics []");
        assert_eq!(
            String::from_wire("#wat.kernel/ProcessPanics []").unwrap(),
            "#wat.kernel/ProcessPanics []"
        );
    }
}

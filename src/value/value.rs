//! The `Value` enum and its full cluster — the central runtime data type.
//!
//! Lifted from `src/runtime.rs` (block ~367–1407) in Stone 251.2e.
//! PURE STRUCTURAL MOVE — no behavior change.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crossbeam_channel;
use chrono;
use holon::HolonAST;
use wat_macros::wat_value;

use crate::ast::WatAST;
use crate::process::ChildHandle;
use crate::hologram::Hologram;
use crate::io::{WatReader, WatWriter};
use crate::rust_deps::{RustOpaqueInner, ThreadOwnedCell};
use crate::channel::{SenderInner, ReceiverInner};
use crate::types::{Nature, TypeExpr};
use crate::value::Function;
use crate::stream::Stream;
use num_bigint::BigInt;
use num_rational::BigRational;

/// Runtime value.
///
/// **Variant names encode their Rust or conceptual origin path via
/// `__` as the namespace separator.** `crossbeam_channel::Sender`
/// becomes `crossbeam_channel__Sender`; only internal `::` is encoded
/// (leading `::` is never written in Rust paths and not encoded here).
/// Prelude types (`bool`, `i64`, `f64`, `String`, `Vec`, `()`) stay
/// short because that's what Rust users write — wat follows Rust's
/// prelude convention.
///
/// `type_name()` returns the full `::`-separated path users write in
/// wat source. Every Value carries its honest identity; error messages
/// say what the user would recognize.
#[wat_value] // Arc 233 Stone 233.2.l: structural seal — forbids wrapping-style variants.
#[derive(Debug, Clone)]
#[allow(non_camel_case_types, non_snake_case)]
pub enum Value {
    bool(bool),
    i64(i64),
    /// `:u8` — unsigned 8-bit integer, 0..=255. Produced by
    /// `:wat::core::u8` (range-checked cast from i64), consumed by
    /// byte-oriented IO (`:wat::io::read`, `:wat::io::write`) and
    /// `:wat::core::Vector<u8>` carriers. Arithmetic is wrapping per Rust's
    /// default u8 semantics. Slice 1 of arc 008.
    u8(u8),
    f64(f64),
    String(Arc<String>),
    /// A `Vec<Value>` — constructed by `:wat::core::vec`.
    Vec(Arc<Vec<Value>>),
    /// The empty tuple / Rust unit `()`. Named `Unit` since `()` isn't
    /// a legal identifier.
    Unit,
    /// Keyword literal — leading `:` included. Wat-source type
    /// `:wat::core::keyword`.
    wat__core__keyword(Arc<String>),
    /// A callable — `define`-registered function or `fn` closure.
    /// Per 058-029: `define` = `fn` + startup-time symbol-table
    /// registration. Static type is `:fn(A,B,...)->R`; the variant
    /// records HOW it was produced.
    wat__core__fn(Arc<Function>),
    /// A composed `wat::holon::HolonAST` — the algebra AST tier carried
    /// at runtime.
    holon__HolonAST(Arc<HolonAST>),
    /// A parsed wat AST carried as a first-class runtime value. Used
    /// by `:wat::eval-ast!` and adjacent forms. Distinct from
    /// [`Value::String`] (raw EDN text that still needs parsing) and
    /// from [`Value::holon__HolonAST`] (algebra AST).
    wat__WatAST(Arc<WatAST>),
    /// A `:wat::kernel::Sender<T>` handle (arc 170 slice 1c).
    /// Carries `Value` — any wat runtime value can travel through.
    /// Transport-polymorphic via [`crate::channel::SenderInner`]:
    /// `Crossbeam` for tier 1 (in-memory), `PipeFd` for tier 2
    /// (EDN-encoded over linux pipes). The user-visible
    /// `Sender<T>` abstraction is uniform across tiers; the
    /// substrate dispatches on the inner enum.
    ///
    /// Pre-arc-170: this variant was named `crossbeam_channel__Sender`
    /// and carried only a crossbeam Sender. Slice 1c renames to the
    /// tier-1/tier-2-honest `wat__kernel__Sender` and adds the
    /// PipeFd transport. The wat-side type alias
    /// `:wat::kernel::Sender<T>` continues to point at this Value
    /// variant; the alias chain to
    /// `:rust::crossbeam_channel::Sender<T>` (in
    /// `wat/kernel/channel.wat`) is preserved for back-compat with
    /// existing user code.
    wat__kernel__Sender(Arc<SenderInner>),
    /// A `:wat::kernel::Receiver<T>` handle (arc 170 slice 1c).
    /// Sibling of [`Self::wat__kernel__Sender`]; same transport
    /// polymorphism shape via
    /// [`crate::channel::ReceiverInner`].
    wat__kernel__Receiver(Arc<ReceiverInner>),
    /// A `:HashMap<K,V>` — Rust std's HashMap natively; stored as
    /// `Arc<HashMap<Value, Value>>` using Stone 216.5a's `impl Hash + PartialEq + Eq
    /// for Value`. No canonical-key crutch; K is the actual HashMap key directly.
    wat__std__HashMap(Arc<HashMap<Value, Value>>),
    /// A `:wat::core::PersistentMap<K,V>` — [`crate::value::pmap::PMap`], the promoting map
    /// (array below `PROMOTION_THRESHOLD`, `rpds::HashTrieMapSync` above it — one-way promotion
    /// on `assoc`). Structural sharing: `assoc`/`dissoc` return a NEW map; the original is
    /// unchanged. Stone arc-278-0a; promoted to `PMap` by DESIGN-STONE-promoting-map.
    wat__core__PersistentMap(crate::value::pmap::PMap),
    /// A `:wat::core::PersistentVector<T>` — rpds `VectorSync<Value>`.
    /// Structural sharing: `conj` (`push_back`) returns a NEW vector (O(log n)); the
    /// original is unchanged. No `Arc` wrapper needed — rpds is already cheap-clone/shared.
    /// Stone arc-278-0b.
    wat__core__PersistentVector(rpds::VectorSync<Value>),
    /// A `:HashSet<T>` — Rust std's HashSet natively; stored as
    /// `Arc<HashSet<Value>>` using Stone 216.5a's `impl Hash + PartialEq + Eq
    /// for Value`. No canonical-key crutch; dedupe via native hash semantics.
    wat__std__HashSet(Arc<HashSet<Value>>),
    /// Generic opaque handle to a Rust-shim-owned value. The
    /// target-form for any `:rust::*` type that doesn't have its own
    /// dedicated Value variant. The inner `RustOpaqueInner` carries a
    /// `type_path` identifier plus an erased payload; shim dispatch
    /// code downcasts via [`crate::rust_deps::downcast_ref_opaque`].
    /// Used by the `#[wat_dispatch]` macro's generated code for all
    /// Self-returning methods.
    RustOpaque(Arc<RustOpaqueInner>),
    /// Abstract byte-source handle — `:wat::io::IOReader`. Wraps any
    /// `WatReader` implementation (real stdin, in-memory `StringIoReader`,
    /// …). Arc 008 slice 2.
    io__IOReader(Arc<dyn WatReader>),
    /// Abstract byte-sink handle — `:wat::io::IOWriter`. Wraps any
    /// `WatWriter` implementation (real stdout/stderr, in-memory
    /// `StringIoWriter`, …). Arc 008 slice 2.
    io__IOWriter(Arc<dyn WatWriter>),
    /// An `:Option<T>` value — `:None` or `(Some v)`. Built-in
    /// parametric enum per 058-030; used as the return type of
    /// `:wat::kernel::recv` / `select` and of structural
    /// retrieval (`get` on HashMap/Vec/HashSet). The `std::option::Option`
    /// here is the Rust host's own Option — wat's `:Option<T>`
    /// compiles to it directly.
    Option(Arc<std::option::Option<Value>>),
    /// A `:Result<T,E>` value — `(Ok v)` or `(Err e)`. Built-in
    /// parametric enum for fallible operations. Surfaced by Rust-dep
    /// shims that wrap crates returning `std::result::Result` (rusqlite
    /// and friends). Constructors are symbol-dispatched (`Ok` / `Err`
    /// as bare identifiers, arity 1 each); consumers use
    /// `(:wat::core::match ...)`.
    Result(Arc<std::result::Result<Value, Value>>),
    /// An `n`-tuple — `:(T1,T2,...,Tn)`. Distinct from [`Value::Vec`]
    /// at the type level (heterogeneous vs homogeneous). Primarily
    /// produced by kernel primitives that return pairs
    /// (`peer-pair'`, `spawn`,
    /// `select`) and destructured in `let` via the
    /// `((a b ...) rhs)` binder shape. The unit type `:()` stays on
    /// [`Value::Unit`] — tuples start at arity 1.
    Tuple(Arc<Vec<Value>>),
    /// A claim-or-panic handle pool — `:HandlePool<T>` per FOUNDATION.
    /// Backing: a bounded crossbeam channel pre-filled with N handles
    /// and its sender dropped immediately, so `is_empty` means the
    /// pool has been fully drained. No Mutex — crossbeam's channel
    /// primitives handle the concurrent `pop` calls lock-free.
    /// `name` surfaces in error messages when a pop from empty or a
    /// finish with orphans fires.
    wat__kernel__HandlePool {
        name: Arc<String>,
        rx: Arc<crossbeam_channel::Receiver<Value>>,
    },
    /// A handle to a child process spawned via
    /// `:wat::kernel::spawn-process` (arc 012 + arc 214). Opaque from
    /// wat's POV — produced by fork. `Drop` SIGKILLs + reaps un-waited
    /// children, keeping zombies out of the process table. Arc-112's
    /// exit-status path (`cached_exit` OnceLock) is the live read;
    /// the retired `:wat::kernel::wait-child` is gone (arc 112/214).
    wat__kernel__ChildHandle(Arc<ChildHandle>),
    /// Arc 293.R2.1 — unified aggregate value (replaces the three former variants:
    /// `Value::Struct`, `Value::wat__core__Record`, `Value::wat__holon__Record`).
    ///
    /// A tagged positional product type. The `nature` field is the ONLY axis of
    /// variance: `{Struct, Record, HolonRecord}` — it gates wire portability,
    /// holon identity, and codegen. The `holon` field is `Empty` for Struct/Record
    /// and `Hologram(h)` for HolonRecord.
    ///
    /// Policy table (enforced by the constructor / macro / runtime, NOT by the repr):
    /// - `Struct`      — never crosses the wire; `holon = Empty`.
    /// - `Record`      — wire-portable (EDN); `holon = Empty`.
    /// - `HolonRecord` — wire-portable + VSA-aligned; `holon = Hologram(h)`.
    ///
    /// `class` is the colon-free FQDN (e.g. `"myapp::Voltage"`) — no leading `:`.
    /// `fields` is the positional field vec in declaration order.
    Aggregate(Arc<AggregateValue>),
    /// An instance of a user-declared `:wat::core::enum` type — a
    /// tagged variant carrying optional positional fields. Arc 048.
    ///
    /// `type_path` is the enum's keyword path (e.g.
    /// `:trading::types::PhaseLabel`); `variant_name` is the variant
    /// identifier (e.g. `"Valley"`). `fields` is empty for unit
    /// variants and populated in declaration order for tagged variants.
    ///
    /// Constructed via:
    /// - Bare keyword `:enum::Variant` — for unit variants. Resolved
    ///   at eval time through `SymbolTable.unit_variants`.
    /// - Invocation `(:enum::Variant arg1 arg2)` — for tagged
    ///   variants. Resolved through an auto-synthesized Function
    ///   entry whose body calls `:wat::core::enum-new`.
    ///
    /// Generic mechanism — covers every user-declared enum.
    /// Built-in `:Option<T>` and `:Result<T,E>` keep their dedicated
    /// `Value::Option` / `Value::Result` variants for substrate-
    /// internal use; user enums use this generic representation.
    Enum(Arc<EnumValue>),
    /// Arc 278 Stone A — `:wat::edn::ForeignRecord`. A self-describing
    /// DYNAMIC record produced by `:wat::edn::read-foreign` on an UNKNOWN
    /// map-bodied tag (`#ns/Type {…}` whose type is not in the registry).
    /// Carries its fully-qualified (colon-free) class + its OWN ordered
    /// key→value fields — SELF-carried, NOT looked up in the registry (the
    /// consumer LACKS the type). Re-serializes faithfully back to the same
    /// `#ns/Type {…}` the reader consumed (round-trip identity). Pure data
    /// (records-are-EDN, arc 300); recursive — a field may itself be a
    /// `ForeignRecord`/`ForeignVariant` decoded all the way down.
    ForeignRecord(Arc<ForeignRecordValue>),
    /// Arc 278 Stone A — `:wat::edn::ForeignVariant`. A self-describing
    /// DYNAMIC enum variant produced by `:wat::edn::read-foreign` on an
    /// UNKNOWN vector-bodied tag (`#<enum-path>/<Variant> [...]`). Carries
    /// the enum class (colon-free FQDN) + variant name + positional fields.
    /// Re-serializes faithfully back to the same `#<enum-path>/<Variant> [...]`.
    /// Sibling of [`Self::ForeignRecord`]; recursive the same way.
    ForeignVariant(Arc<ForeignVariantValue>),
    /// A materialized `:wat::holon::Vector` — the algebra's vector
    /// representation surfaced as a first-class wat value (arc 052).
    /// `Arc` keeps clone cheap (refcount bump only) since vectors at
    /// d=10000 carry 10KB of i8 data.
    ///
    /// Constructed by `:wat::holon::encode <ast>` (explicit
    /// materialization) or by future Vector-tier primitives. Consumed
    /// by polymorphic `cosine` / `dot` / `simhash` (which now accept
    /// Vector or HolonAST in any position) and by Vector-tier ops
    /// shipping in follow-up arcs.
    ///
    /// Equality is bit-exact (element-wise i8 comparison + dim match).
    /// Forced by the Hash + Eq contract for use as HashMap/LruCache
    /// keys. For graded similarity reach for `cosine`, `presence?`,
    /// or `simhash`-then-bucket-then-cosine.
    Vector(Arc<holon::Vector>),
    /// Arc 053 — `:wat::holon::OnlineSubspace`. Incremental PCA that
    /// learns "what normal looks like" from a stream of vectors.
    /// `Arc<ThreadOwnedCell<...>>` for per-thread ownership, zero
    /// Mutex (CSP-safe). Same pattern as wat-lru's LruCache wrapping.
    ///
    /// Mutates via `update`; reads via `residual` / `project` /
    /// `reconstruct` / `eigenvalues`. No equality semantics — two
    /// subspaces trained on different orderings produce different
    /// internal bases.
    OnlineSubspace(Arc<ThreadOwnedCell<holon::OnlineSubspace>>),
    /// Arc 053 — `:wat::holon::Reckoner`. Gradient-trained discriminant
    /// classifier with discrete or continuous readout. Per-thread
    /// owned for safe mutation under CSP.
    Reckoner(Arc<ThreadOwnedCell<holon::Reckoner>>),
    /// Arc 053 — `:wat::holon::Engram`. A learned-pattern snapshot
    /// produced by training. Mostly read-only after construction; the
    /// `residual` method triggers lazy subspace-cache mutation, so we
    /// use ThreadOwnedCell (same per-thread-ownership pattern as the
    /// other state-bearing types). Send+Sync via the same UnsafeCell
    /// + thread-id-check discipline.
    Engram(Arc<ThreadOwnedCell<holon::Engram>>),
    /// Arc 053 — `:wat::holon::EngramLibrary`. The collection-and-
    /// match container for engrams. `Arc<ThreadOwnedCell<...>>` for
    /// per-thread mutation under CSP.
    EngramLibrary(Arc<ThreadOwnedCell<holon::EngramLibrary>>),
    /// Arc 074 slice 1 — `:wat::holon::Hologram<V>`. Coordinate-cell
    /// store with cosine readout, unbounded. The wat-side `<V>` is
    /// phantom — the runtime carries any `Value`. Thread-owned mutable
    /// per `ZERO-MUTEX.md` Tier 2.
    Hologram(Arc<ThreadOwnedCell<Hologram>>),
    /// Arc 056 — `:wat::time::Instant`. Wall-clock point in time
    /// (Java/Clojure lineage; not Rust's monotonic `std::time::Instant`).
    /// Backing: `chrono::DateTime<chrono::Utc>` (Copy + Send + Sync;
    /// no `ThreadOwnedCell` needed). Constructed via
    /// `:wat::time::now`/`at`/`at-millis`/`at-nanos`/`from-iso8601`;
    /// rendered via `to-iso8601`; integer-accessible via
    /// `epoch-seconds`/`epoch-millis`/`epoch-nanos`.
    ///
    /// Arc 056 originally chose no separate Duration type; arc 097
    /// reversed that decision when the lab's debugging UX called for
    /// ActiveSupport-flavored "X ago" composers and `Instant - Instant
    /// → Duration` arithmetic. See [`Value::Duration`].
    Instant(chrono::DateTime<chrono::Utc>),
    /// Arc 097 — `:wat::time::Duration`. Non-negative time interval
    /// expressed in nanoseconds. Distinct runtime variant from
    /// `Value::i64` so polymorphic `:wat::time::-` can dispatch on
    /// the second argument's tag (Instant - Duration → Instant vs
    /// Instant - Instant → Duration, ActiveSupport-shaped).
    /// Non-negative by WAT-surface construction: `time.rs` constructors
    /// panic on negative input and arithmetic panics on negative results.
    /// NOTE: the `i64` storage does not itself enforce this; direct Rust
    /// construction (`Value::Duration(-n)`) bypasses the guard and must
    /// uphold non-negativity as a caller contract. (A future stone makes
    /// this type-enforced via `u64`.)
    /// Constructed via `:wat::time::Hour`/`Minute`/`Second`/`Day`/etc.
    Duration(i64),
    /// Arc 207 — `:wat::core::Uuid`. Typed UUID primitive. Distinct
    /// runtime variant from `Value::String` so `(= some-uuid some-string)`
    /// returns type-mismatch rather than comparing by content — UUIDs are
    /// identifiers, not strings. Pattern B (opaque Value variant) per
    /// `Instant`/`Duration`/`keyword` precedent. `uuid::Uuid` is `Copy`.
    /// Constructed via `Uuid/v4`, `Uuid/v5`, `Uuid/from-string`, `Uuid/nil`.
    wat__core__Uuid(uuid::Uuid),
    /// Arc 220 — `:wat::core::char` (formerly `:wat::core::Char`; Stone 242.1 rename).
    /// Typed character primitive (BMP-only).
    /// Distinct runtime variant from `Value::String` — a char is a single
    /// Unicode scalar value in the BMP (U+0000–U+FFFF), not a string.
    /// Matches wat-edn's `Value::Char` and Clojure's character literal `\c`.
    /// BMP-only inherits Stone 218.6b discipline (supplementary-plane
    /// codepoints U+10000–U+10FFFF rejected at construction + lex time).
    /// Constructed via `(:wat::core::char/of "x")` or `\c` literal.
    wat__core__Char(char),
    /// Arc 300 stone B — `:wat::core::rational` (Stone C1 lowercased the surface;
    /// see the `char` precedent). Typed rational primitive,
    /// REPRESENTATION ONLY (no arithmetic — that is Stone C). Boxed for the
    /// same cache-friendliness reason as other rarely-hot variants.
    /// Always a genuine ratio already reduced to lowest terms with the sign
    /// on the numerator and denominator >= 2 (mirrors `wat-edn::Value::Rational`
    /// / clj's `clojure.lang.Ratio`); a literal reducing to a whole number
    /// (`4/2`) becomes `Value::i64` instead, never this variant.
    /// Constructed via the `<int>/<int>` source literal (`WatAST::RationalLit`).
    wat__core__Rational(Box<BigRational>),
    /// Arc 300 stone C1 — `:wat::core::bigint`. Arbitrary-precision integer,
    /// a FULL first-class arithmetic type (contrast `wat__core__Rational`,
    /// representation-only in Stone B): `+ - *` never wrap/overflow,
    /// contagious (`i64 ⊕ bigint → bigint`, never demotes), `/` collapses
    /// to `bigint` (divisible) or `Rational` (else, reusing `BigRational`).
    /// Boxed for the same cache-friendliness reason as `wat__core__Rational`.
    /// Constructed via the `<int>N` source literal (`WatAST::BigIntLit`).
    wat__core__BigInt(Box<BigInt>),
    /// Arc 220 Stone 220.4 — `:wat::core::List<T>`. Typed linked-list primitive.
    /// Distinct from `Value::Vec` (`:wat::core::Vector`) — preserves the EDN
    /// parens-vs-brackets distinction for faithful round-trips with Clojure.
    /// Backed by `std::collections::LinkedList<Value>`: O(1) cons/head, O(N) iter.
    /// Cross-type equality with Vector per EDN spec §282-289:
    /// `List(1,2,3) == Vector(1,2,3)` returns true.
    /// Hash invariant preserved: List + Vector with same contents hash equal.
    /// conj = PREPEND (Clojure semantic; distinct from Vector conj = APPEND).
    /// Constructed via `(:wat::core::List/of ...)` or `'(...)` literal.
    wat__core__List(std::sync::Arc<std::collections::LinkedList<Value>>),
    /// Arc 118 — `:wat::stream::Stream<T>`. Lazy sequence (Option C: closures + thunks).
    /// SINGLE-PASS — NO memoization (builder, 2026-06-27: *"you cannot walk back a stream …
    /// core does not ship it"*). Diverges from Clojure's persistent lazy-seq: a wat lazy seq
    /// is a stream, consumed once. `(lazy-seq <body>)` captures body unevaluated as a 0-arg
    /// closure; `cons head tail` builds a Cons cell; `seq-empty` is the Empty terminator.
    /// `first`/`rest`/`empty?` force (realize) to WHNF before accessing.
    ///
    /// Equality: pointer identity (Arc::ptr_eq). Structural equality on potentially
    /// infinite seqs is undecidable; pointer equality is the honest bound.
    /// NOT atomizable: cannot appear as HashSet element or HashMap key.
    ///
    /// Constructed by `:wat::stream::empty`, `(:wat::stream::cons h t)`,
    /// `(:wat::stream::lazy <body>)`.
    wat__stream__Stream(Arc<Stream>),
    // Arc 293.R2.1: wat__holon__Record and wat__core__Record DELETED.
    // Both are now represented by Value::Aggregate with nature=HolonRecord/Record.
    // See AggregateValue for the unified repr.
    /// Stone 237.2 — `:wat::core::defclause` multi-arity dispatcher.
    ///
    /// A named set of clauses; each clause has its own arg-type list +
    /// return type + body. Dispatch: arity match first, then arg-type
    /// unification (first-match-wins in declaration order). Consumes
    /// Stone 237.1's typeunion bounded-existential arms transparently at
    /// call time.
    ///
    /// NOT a wrapping variant (carries metadata + Vec<Clause>; not
    /// Arc<Self>). Compiles cleanly under the `#[wat_value]` seal.
    wat__core__clauses(Arc<ClauseSet>),
    /// Arc 232 Stone 232.1 — `:wat::core::extend-type` implementation.
    ///
    /// Carries the type + protocol names and the per-method impl bodies
    /// (parsed as `Clause` values). Stored in `runtime_def_values` under a
    /// canonical `"extend:<T>:<P>"` key. NOT a wrapping variant.
    wat__core__extend_def(Arc<ExtendDef>),
}
// Arc 233 Stone 233.2.k: Value::Tracked variant DELETED.
// Environment now stores TrackedValue directly (Option A); provenance flows
// structurally through the environment without a wrapping variant.
// Stone 233.2.l seals the meta-class via #[wat_value] proc-macro.

// ─── Stone 237.2 — defclause structs ─────────────────────────────────────────

/// Stone 237.2 — one clause of a `defclause` declaration.
///
/// Each clause carries its own arg bindings (name + type), return type,
/// and body AST. The body is evaluated in a child scope that binds the
/// arg names. Produced by parsing a `([name <- :T ...] -> :Ret body)` form.
///
/// Arc 293.4e-pre — `args` is now the canonical `ArgSpec` (replaces the
/// bespoke `Vec<(String,TypeExpr)>` + separate `rest_param` fields).
/// Consumers read `clause.args.fixed_params` (binder names are `Identifier`;
/// `env_key(&id)` where a `String` name is needed) and `clause.args.rest_param`.
#[derive(Debug, Clone)]
pub struct Clause {
    /// Canonical typed-binder-list: fixed positional params + optional rest binder.
    /// Arc 293.4e-pre: replaces the former `args: Vec<(String,TypeExpr)>` +
    /// `rest_param: Option<(String,TypeExpr)>` (which duplicated ArgSpec verbatim).
    pub args: crate::argspec::ArgSpec,
    /// Return type declared for this clause (resolved from shared_return in
    /// Option A, or per-clause `-> :T` in Option B).
    pub return_type: TypeExpr,
    /// Stone 237.3 — Optional `:guard` expression. Evaluated in clause-arg
    /// scope BEFORE the body. `true` → continue; `false` → skip clause.
    /// Runtime error during evaluation propagates.
    pub guard: Option<Arc<WatAST>>,
    /// Stone 237.3 / 237.4 — Optional `:ensure` :fn form. Evaluated AFTER body.
    /// Called with body result; `true` → return result; `false` →
    /// `PostconditionFailed`. Runtime error propagates.
    pub ensure_fn: Option<Arc<WatAST>>,
    /// Body expression. Evaluated in a scope binding the arg names.
    pub body: Arc<WatAST>,
}

/// Stone 237.2 — multi-arity dispatcher container bound to a defclause name.
///
/// `shared_return` carries the top-level `-> :T` from Option A (all clauses
/// share the same return); `None` means each clause declares its own return
/// (Option B, the canonical form). At parse time, Option A sugar is resolved
/// into per-clause return types; `shared_return` is retained only for
/// diagnostics.
#[derive(Debug, Clone)]
pub struct ClauseSet {
    pub name: String,
    pub clauses: Vec<Clause>,
    /// `Some(T)` when the top-level `-> :T` sugar was used (Option A).
    pub shared_return: Option<TypeExpr>,
    /// Optional metadata-map, mirroring `def`/`defn`'s binding-level metadata
    /// (e.g. `{:restricted-to [<prefix-kw>…]}`). `Some(map)` when a `{...}`
    /// form immediately follows the defclause name; `None` when absent.
    ///
    /// Registration stores it into `SymbolTable.binding_metadata` under the
    /// clause's FQDN — exactly as `def`/`defn` do — so the restriction-check
    /// walker (`walk_for_restricted_call` / `extract_prefix_list_from_metadata`
    /// in check.rs) enforces it with no change to the enforcement mechanism.
    pub metadata: Option<HashMap<String, WatAST>>,
}

// ─── end Stone 237.2 structs ──────────────────────────────────────────────────

// ─── Arc 232 Stone 232.1 — extend-type struct ────────────────────────────────

/// Arc 232 Stone 232.1 — an `extend-type` implementation.
///
/// Produced by `parse_extend_type_form`; stored as
/// `Value::wat__core__extend_def` in `runtime_def_values` under a canonical
/// `"extend:<P>:<T>"` key. The `impl_clauses` map holds the parsed impl
/// body for each method (keyed by method name), ready for 232.3 dispatch.
#[derive(Debug, Clone)]
pub struct ExtendDef {
    /// Type FQDN being extended (e.g. `":t::Robot"`).
    pub type_name: String,
    /// Protocol/surface FQDN being implemented — always the BARE name (e.g. `":t::Greeter"`,
    /// or `":probe::Holds"` for `(extend-type :IntBox :Holds<i64> …)`); never carries the
    /// `<...>` type-arg suffix, so lookups against `TypeEnv` (keyed by the bare declared
    /// name) always hit. See `protocol_type_args` for the parsed concrete args.
    pub protocol_name: String,
    /// Arc 170 C2 — the concrete type args from a parametric surface target
    /// (`:Holds<wat::core::i64>` → `[Path(":wat::core::i64")]`). Empty for a monomorphic
    /// surface/protocol target (`:Greeter`) — the common case, a pure no-op.
    pub protocol_type_args: Vec<TypeExpr>,
    /// Per-method impl bodies: method name → `Clause` (argspec + body).
    /// Keyed by method name string. Consumed by 232.3 dispatch.
    pub impl_clauses: HashMap<String, Clause>,
}

// ─── end Arc 232 Stone 232.1 structs ─────────────────────────────────────────

/// Stone 237.4 — per-clause failure reason for defclause dispatch.
///
/// Records WHY a single clause was skipped during defclause dispatch
/// (per arc 233 errors-as-teaching-values doctrine). Each skipped clause
/// contributes one `ClauseAttempt` to the `NoMatchingClause` error.
#[derive(Debug, Clone)]
pub struct ClauseAttempt {
    /// Index of the clause in the defclause declaration (0-based).
    pub clause_index: usize,
    /// Number of parameters declared in this clause.
    pub declared_arity: usize,
    /// Formatted type expression per declared parameter position.
    pub declared_arg_types: Vec<String>,
    /// Why this clause was skipped.
    pub failure_reason: ClauseFailureReason,
}

/// Stone 237.4 — discriminant for why a defclause clause was skipped.
#[derive(Debug, Clone)]
pub enum ClauseFailureReason {
    /// Clause argument count does not match the call's argument count.
    ArityMismatch { expected: usize, got: usize },
    /// A specific argument position's type did not match the declared type.
    ArgTypeMismatch { position: usize, expected: String, got: String },
    /// The clause's `:guard` expression evaluated to `false`.
    GuardFalse,
}

/// Arc 220 Stone 220.4 — shared sequence equality helper.
///
/// Used by `PartialEq for Value` cross-type arms (Vec vs List) and `values_equal`
/// cross-type arms per EDN spec §282-289. Iterates both sequences in lockstep;
/// returns true iff same length and all ordinal pairs are equal.
fn sequence_eq<'a, I, J>(mut a: I, mut b: J) -> bool
where
    I: Iterator<Item = &'a Value>,
    J: Iterator<Item = &'a Value>,
{
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (Some(_), None) | (None, Some(_)) => return false,
            (Some(x), Some(y)) => {
                if x != y {
                    return false;
                }
            }
        }
    }
}

/// Arc 220 Stone 220.4 — shared sequence-Hash helper (β approach).
///
/// Used by `impl Hash for Value` for both `Vec` and `wat__core__List` arms.
/// Both sequences use the same `SEQ_TAG` constant instead of `discriminant`,
/// then hash each element in order. This preserves the Hash invariant:
/// `List(1,2,3) == Vector(1,2,3)` (EDN spec §282-289) → must hash equal.
///
/// `SEQ_TAG` distinguishes sequence-hash inputs from single-discriminant hashes;
/// correctness rests on the full 64-bit hash output (collision ~1/2^64), not on
/// integer-discriminant ranges (`Value` is data-carrying, so `std::mem::discriminant`
/// hashes as an opaque `Discriminant<T>`, not a raw int).
fn hash_sequence<'a, H, I>(items: I, state: &mut H)
where
    H: std::hash::Hasher,
    I: IntoIterator<Item = &'a Value>,
{
    use std::hash::Hash;
    const SEQ_TAG: u8 = 0xA5;
    SEQ_TAG.hash(state);
    for v in items {
        v.hash(state);
    }
}

/// Arc 216 Stone 216.5a — `impl PartialEq for Value`.
///
/// Manual impl mirroring `HolonAST`'s pattern (`holon-rs/src/kernel/holon_ast.rs:158-192`).
/// Structural per-variant; f64 via `to_bits()` (NaN-safe). Non-atomizable variants
/// use `Arc::ptr_eq` where identity is pointer-based (opaque handles, ML types, IO
/// handles); the cross-variant arm returns `false`.
///
/// ## Variant classification (per the `is_atomizable` predicate in `src/check.rs`)
///
/// **Atomizable** (may appear as HashSet elements / HashMap keys):
/// `bool`, `i64`, `f64`, `String`, `wat__core__keyword`, `holon__HolonAST`,
/// `wat__WatAST`, `wat__core__Uuid`, `wat__core__Char`, `Aggregate` (Record/HolonRecord),
/// `Unit` (`:wat::core::nil`), `Vec` (recursive),
/// `wat__std__HashSet` (recursive), `wat__std__HashMap` (recursive),
/// `Tuple` (iff all element types atomizable).
///
/// **Structurally-equal but NOT atomizable** (natural equality; not predicate-admitted):
/// `u8`, `Option`, `Result`, `Aggregate(Struct)`, `Enum`, `Vector` (holon::Vector),
/// `Instant`, `Duration`, `wat__core__List` (not in `is_atomizable`).
///
/// **Opaque handles** (pointer equality; not atomizable; never in HashSet/HashMap keys):
/// `wat__core__fn`, `wat__core__clauses` (pointer-equality like fn),
/// `wat__kernel__Sender`, `wat__kernel__Receiver`,
/// `wat__kernel__HandlePool`, `wat__kernel__ChildHandle`,
/// `RustOpaque`, `io__IOReader`, `io__IOWriter`,
/// `OnlineSubspace`, `Reckoner`, `Engram`, `EngramLibrary`, `Hologram`.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use std::sync::Arc;
        match (self, other) {
            // --- Atomizable primitives ---
            (Value::bool(a), Value::bool(b)) => a == b,
            (Value::i64(a), Value::i64(b)) => a == b,
            (Value::f64(a), Value::f64(b)) => a.to_bits() == b.to_bits(),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::wat__core__keyword(a), Value::wat__core__keyword(b)) => a == b,
            (Value::holon__HolonAST(a), Value::holon__HolonAST(b)) => a == b,
            (Value::wat__WatAST(a), Value::wat__WatAST(b)) => a == b,
            (Value::wat__core__Uuid(a), Value::wat__core__Uuid(b)) => a == b,
            // Arc 220 — Char equality. `char` implements `PartialEq`.
            (Value::wat__core__Char(a), Value::wat__core__Char(b)) => a == b,
            // Arc 300 stone B — Rational equality. `BigRational` implements
            // `PartialEq` (structural, already-reduced so no `1/2 != 2/4` gap).
            (Value::wat__core__Rational(a), Value::wat__core__Rational(b)) => a == b,
            // Arc 300 stone C1 — BigInt equality. `num_bigint::BigInt` implements
            // `PartialEq` (structural). Category-aware cross-type equality with
            // i64 (both INTEGER category) lives here too — `values_equal` in
            // runtime.rs is the polymorphic `=` entry point and mirrors this arm;
            // this `Value::eq` (structural Rust equality, used for HashMap/HashSet
            // keys where BigInt is NOT atomizable) stays same-type-only.
            (Value::wat__core__BigInt(a), Value::wat__core__BigInt(b)) => a == b,
            // Arc 220 Stone 220.4 — List same-type equality.
            (Value::wat__core__List(a), Value::wat__core__List(b)) => {
                sequence_eq(a.iter(), b.iter())
            }
            // Arc 220 Stone 220.4 — Cross-type sequence equality per EDN spec §282-289.
            // `List(1,2,3) == Vector(1,2,3)` returns true; order + contents match.
            (Value::Vec(a), Value::wat__core__List(b)) => sequence_eq(a.iter(), b.iter()),
            (Value::wat__core__List(a), Value::Vec(b)) => sequence_eq(a.iter(), b.iter()),
            (Value::Vec(a), Value::Vec(b)) => a == b,
            // HashSet: native Arc<HashSet<Value>> equality.
            // Stone 216.5b — storage is now Arc<HashSet<Value>>; HashSet's PartialEq
            // impl delegates to element PartialEq (order-independent set semantics).
            // Reduces to a single native comparison.
            (Value::wat__std__HashSet(a), Value::wat__std__HashSet(b)) => a == b,
            // HashMap: Stone 216.5c — native Arc<HashMap<Value,Value>> equality.
            // std HashMap PartialEq uses Value's PartialEq on both K and V.
            // Reduces to a single native comparison.
            (Value::wat__std__HashMap(a), Value::wat__std__HashMap(b)) => a == b,
            // PersistentMap: rpds::HashTrieMapSync implements PartialEq — delegate.
            // Arc-278-0a: structural equality over K/V using Value's PartialEq.
            (Value::wat__core__PersistentMap(a), Value::wat__core__PersistentMap(b)) => a == b,
            // PersistentVector: rpds::VectorSync implements PartialEq — delegate.
            // Arc-278-0b: order-dependent equality (a vector's order is semantic).
            (Value::wat__core__PersistentVector(a), Value::wat__core__PersistentVector(b)) => a == b,
            // --- Structurally-equal but NOT atomizable ---
            (Value::u8(a), Value::u8(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Option(a), Value::Option(b)) => a == b,
            (Value::Result(a), Value::Result(b)) => a == b,
            // Arc 294.c.1 — identity is the EDN data (Q-D): (nature, class, fields).
            // The hologram is a DERIVED index, never identity. Collapses flaw #7 — the
            // equality split-brain; aligns with values_equal in runtime.rs.
            // > SUPERSEDED 2026-06-28 by arc 294.c.1: prior contract keyed on hologram
            // > (arc 293.R2.1 "identity lives in hologram") — replaced by data identity.
            (Value::Aggregate(a), Value::Aggregate(b)) => {
                // Arc 294.c.1 — identity is the EDN data (Q-D): (nature, class, fields).
                // The hologram is a DERIVED index, never identity (collapses flaw #7 — the
                // equality split-brain; aligns with values_equal in runtime.rs).
                a.nature == b.nature && a.class == b.class && a.fields == b.fields
            }
            (Value::Enum(a), Value::Enum(b)) => {
                a.type_path == b.type_path
                    && a.variant_name == b.variant_name
                    && a.fields == b.fields
            }
            // Arc 278 Stone A — foreign dynamic values: structural identity on
            // the self-carried data (class + ordered key→value fields / enum-class
            // + variant + positional fields). Pure data (records-are-EDN).
            (Value::ForeignRecord(a), Value::ForeignRecord(b)) => {
                a.class == b.class && a.fields == b.fields
            }
            (Value::ForeignVariant(a), Value::ForeignVariant(b)) => {
                a.enum_class == b.enum_class
                    && a.variant == b.variant
                    && a.fields == b.fields
            }
            // holon::Vector: bit-exact (PartialEq impl in holon-rs compares data slices)
            (Value::Vector(a), Value::Vector(b)) => a == b,
            // chrono::DateTime implements PartialEq
            (Value::Instant(a), Value::Instant(b)) => a == b,
            // Duration is stored as i64 nanoseconds
            (Value::Duration(a), Value::Duration(b)) => a == b,
            // --- Opaque handles: pointer equality ---
            // These are never atomizable; pointer identity is the only meaningful equality.
            (Value::wat__core__fn(a), Value::wat__core__fn(b)) => Arc::ptr_eq(a, b),
            (Value::wat__kernel__Sender(a), Value::wat__kernel__Sender(b)) => Arc::ptr_eq(a, b),
            (Value::wat__kernel__Receiver(a), Value::wat__kernel__Receiver(b)) => {
                Arc::ptr_eq(a, b)
            }
            (Value::wat__kernel__HandlePool { rx: a, .. }, Value::wat__kernel__HandlePool { rx: b, .. }) => {
                Arc::ptr_eq(a, b)
            }
            (Value::wat__kernel__ChildHandle(a), Value::wat__kernel__ChildHandle(b)) => {
                Arc::ptr_eq(a, b)
            }
            (Value::RustOpaque(a), Value::RustOpaque(b)) => Arc::ptr_eq(a, b),
            // dyn trait objects: pointer equality on the data pointer
            (Value::io__IOReader(a), Value::io__IOReader(b)) => Arc::ptr_eq(a, b),
            (Value::io__IOWriter(a), Value::io__IOWriter(b)) => Arc::ptr_eq(a, b),
            // ML types: per-thread-owned; pointer identity is the only meaningful equality
            (Value::OnlineSubspace(a), Value::OnlineSubspace(b)) => Arc::ptr_eq(a, b),
            (Value::Reckoner(a), Value::Reckoner(b)) => Arc::ptr_eq(a, b),
            (Value::Engram(a), Value::Engram(b)) => Arc::ptr_eq(a, b),
            (Value::EngramLibrary(a), Value::EngramLibrary(b)) => Arc::ptr_eq(a, b),
            (Value::Hologram(a), Value::Hologram(b)) => Arc::ptr_eq(a, b),
            // Arc 293.R2.1: wat__holon__Record and wat__core__Record arms removed; handled by Aggregate above.
            // Stone 237.2 — wat__core__clauses: pointer equality (two ClauseSet instances
            // are the same dispatcher iff they are the same Arc). Structural equality
            // over clause bodies is not implemented — same rationale as wat__core__fn.
            (Value::wat__core__clauses(a), Value::wat__core__clauses(b)) => Arc::ptr_eq(a, b),
            // Arc 118 — Stream: pointer equality. Structural equality on potentially infinite
            // seqs is undecidable; pointer identity is the honest bound (STOP-1 avoided).
            (Value::wat__stream__Stream(a), Value::wat__stream__Stream(b)) => Arc::ptr_eq(a, b),
            // Cross-variant pairs are always unequal
            _ => false,
        }
    }
}

/// Arc 216 Stone 216.5a — `impl Eq for Value`.
/// Marker trait. Safe per NaN-bit-pattern equality in `PartialEq::eq`.
impl Eq for Value {}

/// Arc 216 Stone 216.5a — `impl Hash for Value`.
///
/// Mirrors `HolonAST`'s pattern (`holon-rs/src/kernel/holon_ast.rs:196-232`):
/// `std::mem::discriminant` tagging first (prevents `bool(true) == i64(1)`
/// cross-variant collisions), then per-variant payload hashing.
///
/// f64 uses `to_bits()` (NaN-safe). Recursive variants (HashSet, HashMap, Vec,
/// HolonAST, WatAST) compose for free via std lib's `Hash` impls.
///
/// **HashSet/HashMap arms bypass the canonical-key crutch entirely** (BRIEF § Part C):
/// - `HashSet`: collect element hash values as `u64`s, sort, hash the sorted list.
///   Sort-then-hash gives determinism (set semantics: {a,b} == {b,a} → same hash).
/// - `HashMap`: collect (key_hash, val_hash) pairs as `(u64,u64)`, sort by key_hash,
///   hash the sorted list. Map semantics: {a→1, b→2} == {b→2, a→1} → same hash.
///
/// **Non-atomizable variants → `unreachable!()`** with predicate-citation message.
/// The `is_atomizable` predicate at `src/check.rs` is the static guarantee that
/// only atomizable Values reach hashing contexts (HashSet/HashMap key positions).
/// If this panic ever fires, the predicate has drifted from the Hash impl.
///
/// **Structural-but-not-atomizable variants** (`u8`, `Unit`, `Tuple`, `Option`,
/// `Result`, `Aggregate(Struct)`, `Enum`, `Vector`, `Instant`, `Duration`) receive structural
/// Hash impls rather than `unreachable!()`. Per STOP-4: these variants ARE reachable
/// in Rust code (e.g., as HashMap values or as elements of an outer Tuple) and have
/// well-defined structural hash semantics. They are NOT currently atomizable (not in
/// the `is_atomizable` predicate), but their hash implementations are honest. If the
/// predicate is later extended to admit them, the Hash impl is already
/// correct.
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Arc 220 Stone 220.4 — sequence types (Vec + List) skip the
        // global discriminant and use hash_sequence (shared SEQ_TAG + element
        // iteration) so that `List(1,2,3)` and `Vector(1,2,3)` hash equal per
        // EDN spec §282-289 (cross-type sequence equality). Early-return before
        // the discriminant fires for these two variants only.
        match self {
            Value::Vec(xs) => return hash_sequence(xs.iter(), state),
            Value::wat__core__List(xs) => return hash_sequence(xs.iter(), state),
            _ => {}
        }
        std::mem::discriminant(self).hash(state);
        match self {
            // --- Atomizable primitives ---
            Value::bool(b) => b.hash(state),
            Value::i64(n) => n.hash(state),
            Value::f64(x) => x.to_bits().hash(state),
            Value::String(s) => s.hash(state),
            Value::wat__core__keyword(k) => k.hash(state),
            Value::holon__HolonAST(h) => h.hash(state),
            Value::wat__WatAST(ast) => ast.hash(state),
            Value::wat__core__Uuid(u) => u.hash(state),
            // Arc 220 — Char hash. `char` implements `Hash`.
            Value::wat__core__Char(c) => c.hash(state),
            // Arc 300 stone B — Rational hash. `BigRational` implements `Hash`.
            Value::wat__core__Rational(r) => r.hash(state),
            // Arc 300 stone C1 — BigInt hash. `num_bigint::BigInt` implements `Hash`.
            Value::wat__core__BigInt(n) => n.hash(state),
            // Arc 220 Stone 220.4 — Vec + List handled above (early-return); unreachable.
            Value::Vec(_) | Value::wat__core__List(_) => unreachable!("handled above"),
            // HashSet: sort element hashes for set semantics (order-independent).
            // Stone 216.5b — storage is now Arc<HashSet<Value>>; iterate s.iter()
            // directly (Values, not String canonical-keys).
            Value::wat__std__HashSet(s) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::Hasher;
                let mut elem_hashes: Vec<u64> = s.iter().map(|v| {
                    let mut h = DefaultHasher::new();
                    v.hash(&mut h);
                    h.finish()
                }).collect();
                elem_hashes.sort_unstable();
                elem_hashes.hash(state);
            }
            // HashMap: sort (key_hash, val_hash) pairs for map semantics (order-independent).
            // Stone 216.5c — iterate m.iter() for (k, v) directly (no canonical-key tuple).
            Value::wat__std__HashMap(m) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::Hasher;
                let mut pair_hashes: Vec<(u64, u64)> = m.iter().map(|(k, v)| {
                    let mut kh = DefaultHasher::new();
                    k.hash(&mut kh);
                    let mut vh = DefaultHasher::new();
                    v.hash(&mut vh);
                    (kh.finish(), vh.finish())
                }).collect();
                pair_hashes.sort_unstable();
                pair_hashes.hash(state);
            }
            // PersistentMap: order-independent hash. `PMap`'s own `Hash` impl is this exact
            // sorted-pair routine, moved into pmap.rs — one routine now covers both arms.
            Value::wat__core__PersistentMap(m) => {
                m.hash(state);
            }
            // PersistentVector: order-DEPENDENT hash — sequence hash over elements in order.
            // Arc-278-0b: a vector's order is semantic; discriminant already hashed above;
            // hash each element in order (mirrors std Vec's hash_sequence semantics, but with
            // its own discriminant so it hashes DISTINCT from Vec / List).
            Value::wat__core__PersistentVector(v) => {
                for elem in v.iter() {
                    elem.hash(state);
                }
            }
            // --- Structural but NOT atomizable: honest hash impls (STOP-4 surface) ---
            Value::u8(n) => n.hash(state),
            Value::Unit => {
                // Unit has no payload; discriminant alone is the hash
            }
            Value::Tuple(xs) => xs.hash(state),
            Value::Option(opt) => match opt.as_ref() {
                None => 0u8.hash(state),
                Some(v) => {
                    1u8.hash(state);
                    v.hash(state);
                }
            },
            Value::Result(res) => match res.as_ref() {
                Ok(v) => {
                    0u8.hash(state);
                    v.hash(state);
                }
                Err(e) => {
                    1u8.hash(state);
                    e.hash(state);
                }
            },
            // Arc 294.c.1 — hash on the EDN data (nature, class, fields), matching PartialEq.
            // The hologram is a derived index and is NOT part of identity (flaw #7 collapse).
            // > SUPERSEDED 2026-06-28 by arc 294.c.1: prior contract hashed the hologram
            // > (arc 293.R2.1 "Hologram → hash on hologram (canonical identity, Stone 234.1)").
            Value::Aggregate(a) => {
                // Stamped at construction for shallow payloads (facts). 0 means
                // nested collections (Session) — walk, do not O(n²) at insert.
                if a.identity != 0 {
                    a.identity.hash(state);
                } else {
                    a.nature.hash(state);
                    a.class.hash(state);
                    a.fields.hash(state);
                }
            }
            Value::Enum(e) => {
                e.type_path.hash(state);
                e.variant_name.hash(state);
                e.fields.hash(state);
            }
            // Arc 278 Stone A — foreign dynamic values: honest structural hash on
            // the self-carried data (matches PartialEq). Pure data, so hashing is
            // well-defined; kept consistent with Aggregate/Enum discipline.
            Value::ForeignRecord(a) => {
                a.class.hash(state);
                a.fields.hash(state);
            }
            Value::ForeignVariant(a) => {
                a.enum_class.hash(state);
                a.variant.hash(state);
                a.fields.hash(state);
            }
            // holon::Vector: hash the underlying i8 data slice
            Value::Vector(v) => v.data().hash(state),
            // chrono::DateTime<Utc>: hash via timestamp_nanos (i64, unique per instant)
            Value::Instant(dt) => dt.timestamp_nanos_opt().hash(state),
            // Duration: stored as i64 nanoseconds
            Value::Duration(ns) => ns.hash(state),
            // --- Non-atomizable variants: unreachable!() with predicate citation ---
            // The is_atomizable predicate at src/check.rs is the static guarantee
            // that these variants never reach hashing contexts (HashSet/HashMap key positions).
            // If this panic fires, the predicate has drifted from this Hash impl.
            // rune:coverage(unreachable) [cluster] — is_atomizable (src/check.rs) statically gates every non-atomizable variant out of all hashing contexts before this match; each arm below is provably dead until coverage resumes, and the panic IS the bug if one ever fires (cf. the predicate-drift note above).
            Value::wat__core__fn(_) => unreachable!(
                "Value::wat__core__fn is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::wat__kernel__Sender(_) => unreachable!(
                "Value::wat__kernel__Sender is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::wat__kernel__Receiver(_) => unreachable!(
                "Value::wat__kernel__Receiver is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::wat__kernel__HandlePool { .. } => unreachable!(
                "Value::wat__kernel__HandlePool is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::wat__kernel__ChildHandle(_) => unreachable!(
                "Value::wat__kernel__ChildHandle is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::RustOpaque(_) => unreachable!(
                "Value::RustOpaque is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::io__IOReader(_) => unreachable!(
                "Value::io__IOReader is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::io__IOWriter(_) => unreachable!(
                "Value::io__IOWriter is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::OnlineSubspace(_) => unreachable!(
                "Value::OnlineSubspace is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::Reckoner(_) => unreachable!(
                "Value::Reckoner is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::Engram(_) => unreachable!(
                "Value::Engram is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::EngramLibrary(_) => unreachable!(
                "Value::EngramLibrary is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            Value::Hologram(_) => unreachable!(
                "Value::Hologram is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
            // Arc 293.R2.1: wat__holon__Record and wat__core__Record Hash arms removed; handled by Aggregate above.
            // Stone 237.2 — wat__core__clauses: hash via Arc pointer (consistent with
            // pointer-equality PartialEq). Same discipline as wat__core__fn.
            Value::wat__core__clauses(cs) => {
                "wat__core__clauses".hash(state);
                (Arc::as_ptr(cs) as usize).hash(state);
            }
            // Arc 232 Stone 232.1 — registry carriers: not atomizable; pointer hash.
            Value::wat__core__extend_def(ed) => {
                "wat__core__extend_def".hash(state);
                (Arc::as_ptr(ed) as usize).hash(state);
            }
            // Arc 118 — Stream: not atomizable (infinite seqs make hashing undecidable).
            // The is_atomizable predicate in src/check.rs is the static guarantee.
            Value::wat__stream__Stream(_) => unreachable!(
                "Value::wat__stream__Stream is not atomizable; is_atomizable predicate in \
                 src/check.rs should have rejected this. If you see this panic, \
                 the predicate has drifted."
            ),
        }
    }
}

/// Arc 293.R2.1 — hologram presence/absence for an aggregate value.
///
/// `Empty` for `Nature::Struct` and `Nature::Record` (no VSA dual-form).
/// `Hologram(h)` for `Nature::HolonRecord` (the canonical VSA identity form).
///
/// Named enum (NOT `Option`) per `feedback_option_carrying_semantics_screams_enum`:
/// `None` would overload "absence" with the identity-gating semantic; `Empty` names
/// the thing honestly and makes the illegal state (`Hologram` on a non-holon nature)
/// representable but policy-rejected at construction time.
#[derive(Debug, Clone)]
pub enum HolonForm {
    /// No hologram — structural identity only (`Nature::Struct` / `Nature::Record`).
    Empty,
    /// VSA canonical form — `Nature::HolonRecord` identity lives here.
    Hologram(Arc<HolonAST>),
}

/// Arc 293.R2.1 — unified product-type value payload.
///
/// Replaces `StructValue` + the three inline record variant payloads
/// (the old `Value::Struct`, `Value::wat__core__Record`, `Value::wat__holon__Record` — all gone).
///
/// `class` is the COLON-FREE FQDN (e.g. `"myapp::Voltage"` — no leading `:`).
/// `fields` is the positional field vec in declaration order.
/// `nature` is the categorical axis (`{Struct, Record, HolonRecord}`).
/// `holon` is `Empty` for Struct/Record and `Hologram(h)` for HolonRecord.
#[derive(Clone)]
pub struct AggregateValue {
    /// Colon-free FQDN of the declared type (e.g. `"wat::kernel::Process"`).
    /// Was `StructValue.type_name` (stripped of leading `:`) /
    /// `wat__core__Record.class_fqdn` / `wat__holon__Record.class_fqdn`.
    pub class: String,
    /// Field names in declaration order. **Same length as `fields`, always.**
    /// Arc 296 G: carried, never looked up — see the sibling `Value::ForeignRecord`,
    /// which self-carries its keys and has never had the `field-N` bug.
    pub names: Arc<Vec<String>>,
    /// Positional field values in declaration order.
    /// Was `StructValue.fields` (wrapped in Arc) / the old `struct_form` local name.
    pub fields: Arc<Vec<Value>>,
    /// The categorical label: `{Struct, Record, HolonRecord}`.
    pub nature: Nature,
    /// `Empty` for Struct/Record; `Hologram(h)` for HolonRecord.
    pub holon: HolonForm,
    /// FxHash of `(nature, class, fields)` — Hash cache, not EDN.
    /// `DESIGN-STONE-aggregate-identity`. Private so construction must restamp.
    identity: u64,
}

fn value_is_shallow(v: &Value) -> bool {
    match v {
        Value::bool(_)
        | Value::i64(_)
        | Value::u8(_)
        | Value::f64(_)
        | Value::String(_)
        | Value::Unit
        | Value::wat__core__keyword(_)
        | Value::wat__core__Char(_)
        | Value::wat__core__Uuid(_)
        | Value::Instant(_)
        | Value::Duration(_) => true,
        Value::Aggregate(a) => a.identity != 0,
        Value::Enum(e) => e.fields.iter().all(value_is_shallow),
        _ => false,
    }
}

impl fmt::Debug for AggregateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AggregateValue")
            .field("class", &self.class)
            .field("names", &self.names)
            .field("fields", &self.fields)
            .field("nature", &self.nature)
            .field("holon", &self.holon)
            .finish()
    }
}

impl AggregateValue {
    /// Single construction funnel. Stamps `identity` (`DESIGN-STONE-aggregate-identity`).
    pub(crate) fn from_parts(
        class: String,
        names: Arc<Vec<String>>,
        fields: Arc<Vec<Value>>,
        nature: Nature,
        holon: HolonForm,
    ) -> Self {
        // Stamp only a shallow payload. A Session's fields include the facts
        // PV — hashing that at every insert is O(n²). identity 0 → Hash walks
        // (`DESIGN-STONE-aggregate-identity`).
        let identity = if fields.iter().all(value_is_shallow) {
            let mut h = rustc_hash::FxHasher::default();
            nature.hash(&mut h);
            class.hash(&mut h);
            fields.hash(&mut h);
            let id = h.finish();
            if id == 0 { 1 } else { id }
        } else {
            0
        };
        Self {
            class,
            names,
            fields,
            nature,
            holon,
            identity,
        }
    }

    /// Construct a Struct-nature aggregate (no hologram).
    /// `class` must be WITHOUT the leading colon.
    pub fn struct_(class: String, names: Arc<Vec<String>>, fields: Vec<Value>) -> Self {
        Self::from_parts(class, names, Arc::new(fields), Nature::Struct, HolonForm::Empty)
    }
    /// Construct a base-Record aggregate (no hologram).
    pub fn record(class: String, names: Arc<Vec<String>>, fields: Arc<Vec<Value>>) -> Self {
        Self::from_parts(class, names, fields, Nature::Record, HolonForm::Empty)
    }
    /// Construct a HolonRecord aggregate (with hologram).
    pub fn holon_record(class: String, names: Arc<Vec<String>>, fields: Arc<Vec<Value>>, hologram: Arc<HolonAST>) -> Self {
        Self::from_parts(
            class,
            names,
            fields,
            Nature::HolonRecord,
            HolonForm::Hologram(hologram),
        )
    }
}

/// Arc 296 G — shared by every class-C construction site (a statically-known wat-declared
/// type, no registry in scope): turns a `&'static [&'static str]` const emitted by
/// `wat_field_names_from!` into an `Arc<Vec<String>>`. Callers cache the result behind their
/// own `OnceLock` so a hot path (e.g. raising a `Fault`) allocates the vec once, not per call.
pub(crate) fn names_arc_from_static(fields: &'static [&'static str]) -> Arc<Vec<String>> {
    Arc::new(fields.iter().map(|s| (*s).to_string()).collect())
}

// Arc 293.R2.1: StructValue ANNIHILATED — replaced by AggregateValue.
// Tombstone: old `StructValue { type_name: String, fields: Vec<Value> }`.
// `AggregateValue.class` is colon-free; `fields` is `Arc<Vec<Value>>`.

/// The payload of a [`Value::Enum`] — the enum's fully-qualified
/// declared type path, the variant identifier, and the variant's
/// positional field values (empty for unit variants). Arc 048.
///
/// `type_path` matches the enum's declared name verbatim
/// (`:trading::types::PhaseLabel`); `variant_name` is the variant's
/// identifier without the path prefix (`Valley`).
#[derive(Debug, Clone)]
pub struct EnumValue {
    pub type_path: String,
    pub variant_name: String,
    /// Field names in declaration order. **Same length as `fields`, always.**
    /// Arc 296 G′: carried, never looked up — the enum mirror of `AggregateValue.names`.
    pub names: Arc<Vec<String>>,
    pub fields: Vec<Value>,
}

/// Arc 278 Stone A — payload of a [`Value::ForeignRecord`].
///
/// A self-describing dynamic record: the fully-qualified (COLON-FREE) class
/// (e.g. `"some::unknown::Rec"`) and its OWN ordered key→value fields. The
/// keys are self-carried (the bare keyword name, e.g. `"kind"`) rather than
/// looked up in a type registry — a `read-foreign` consumer LACKS the type,
/// so the wire form is the only source of the field names. Order is preserved
/// as read so re-serialization reproduces the exact `#ns/Type {…}` body.
#[derive(Debug, Clone)]
pub struct ForeignRecordValue {
    /// Colon-free fully-qualified class (e.g. `"some::unknown::Rec"`).
    pub class: String,
    /// Ordered (bare-keyword-name → value) fields, self-carried from the wire.
    pub fields: Vec<(String, Value)>,
}

/// Arc 278 Stone A — payload of a [`Value::ForeignVariant`].
///
/// A self-describing dynamic enum variant: the enum's colon-free FQDN
/// (`"some::unknown::Kind"`), the variant name (`"Click"`), and the
/// positional field values (recursively decoded). Re-serializes to the same
/// `#<enum-path>/<Variant> [...]` the reader consumed.
#[derive(Debug, Clone)]
pub struct ForeignVariantValue {
    /// Colon-free fully-qualified enum class (e.g. `"some::unknown::Kind"`).
    pub enum_class: String,
    /// Variant name without path prefix (e.g. `"Click"`).
    pub variant: String,
    /// Positional field values in wire order.
    pub fields: Vec<Value>,
}

// ─── BRIEF-key-eligibility-wall — the wall: "interior-mutable AND hashable" ──
// ─── is unrepresentable ───────────────────────────────────────────────────

/// Whether a `Value` variant may be used as a hash key.
///
/// The wall: there is deliberately NO way to spell "carries interior mutability AND is
/// hashable". That state has no constructor, so it cannot be written down — which is the
/// point of this type existing at all rather than a bare `bool`. A `bool` would let someone
/// write `true` for `Sender`; this shape means the wrong classification is unrepresentable,
/// and the only way to mark something hashable is to assert it is pure data.
///
/// Ground truth for every variant is read off `impl Hash for Value` (the `unreachable!()`
/// arms with their predicate-citation messages) and `impl PartialEq for Value` (the
/// `Arc::ptr_eq` arms). See `key_eligibility()`, the exhaustive sibling of `type_name()`
/// that assigns this per variant, and `all_key_eligibility()`, the gate-testable table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEligibility {
    /// Pure data. May be a key; `is_atomizable` MUST accept this variant's checker-facing
    /// type.
    Hashable,
    /// Never a key. Its `Hash` arm is `unreachable!()` (or, for the recursive containers,
    /// its checker type is rejected) and `is_atomizable` MUST reject it.
    NeverAKey(NotAKeyReason),
}

/// Why a `Value` variant is `KeyEligibility::NeverAKey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAKeyReason {
    /// Carries interior mutability (an `AtomicBool`, `UnsafeCell`, `OnceLock`, …) reachable
    /// from the value, so a hash taken now may not hold later. Verified by reading the
    /// wrapped type's own field list, not merely by its `Hash` arm — `wat__kernel__Sender`
    /// (`SenderInner::Comms.closed: AtomicBool`), `wat__kernel__ChildHandle`
    /// (`ChildHandle.reaped: AtomicBool` + `cached_exit: OnceLock<i64>`), and the five
    /// `Arc<ThreadOwnedCell<_>>` ML types (`ThreadOwnedCell.cell: UnsafeCell<T>`, directly).
    InteriorMutable,
    /// An opaque handle whose identity is pointer-based (`Arc::ptr_eq`), not structural —
    /// no interior-mutable field is provable from this crate's own struct definitions
    /// (a `dyn Trait` payload is opaque by construction; a callable's identity is
    /// intentionally pointer-based regardless of what its closed environment holds).
    OpaqueHandle,
    /// Structurally hashable in principle — the `Hash` arm is real, not `unreachable!()` —
    /// but `is_atomizable` does not currently admit it. Covers both "deliberately excluded"
    /// (arc 216 Stone 7's Tuple sibling `wat__core__List`, kept off the list on purpose per
    /// its own doc comment) and "not yet taught to the checker" (`PersistentMap`/
    /// `PersistentVector` — no `Parametric` arm in `is_atomizable` at all, unlike
    /// `HashMap`/`HashSet`/`Vector`, which share the mechanism).
    ExcludedByDesign,
}

/// Expands one row's `gate:` list into `(TypeExpr, KeyEligibility)` pairs.
///
/// **Two forms, and the DEFAULT is the one that cannot drift:**
///
/// - `gate: [ ty, ty ]` — each probe is paired with the row's own `key_eligibility`, so the
///   gate necessarily asserts what `Value::key_eligibility()` actually returns. Flipping a
///   row's classification moves both, and the gate goes red. **44 of 46 rows use this.**
/// - `gate: [ ty => ke, … ]` — per-probe eligibility, for the two rows whose
///   `key_eligibility` expression **binds pattern variables** (`Aggregate` matches on
///   `a.nature`; `RustOpaque` reads `inner`) and therefore cannot be evaluated in the static
///   table at all. Their probes are independent by necessity, not by convenience.
///
/// Reach for the second form ONLY when the first fails to compile because the row's
/// eligibility is not pattern-independent. Using it to hand-write a probe that disagrees
/// with the row's own classification re-opens exactly the drift this shape closes — a
/// deliberate breach (flip a row to `Hashable`, watch the gate stay green) is how that gap
/// was found in the first place.
macro_rules! ke_gate_entries {
    ($ke:expr, [ $( $ty:expr ),+ $(,)? ]) => { vec![ $( ($ty, $ke) ),+ ] };
    ($ke:expr, [ $( $ty:expr => $gke:expr ),+ $(,)? ]) => { vec![ $( ($ty, $gke) ),+ ] };
}

/// Emits `Value::type_name()`, `Value::key_eligibility()`, and
/// `Value::all_key_eligibility()` from ONE list — one row per `Value` variant — so a variant
/// cannot appear in the type-name match and be missing from the key-eligibility match (STOP-2:
/// two independently hand-maintained matches is a convention that can drift; one macro list is
/// the wall). Each row supplies:
///
/// - `$pat` — the match pattern (reused verbatim in both matches).
/// - `type_name` — `type_name()`'s existing arm body, moved here unchanged (PURE STRUCTURAL
///   MOVE — no behavior change; see arc-109 BRIEF-key-eligibility-wall.md Room 1).
/// - `key_eligibility` — the classification, read off the `Hash`/`PartialEq` ground truth.
/// - `gate` — one or more `(TypeExpr, KeyEligibility)` checker-probe pairs. Almost always a
///   single pair mirroring `type_name`/`key_eligibility` (with the type spelled as the
///   `:`-prefixed path `is_atomizable` actually matches against — see `TypeExpr::Path`'s own
///   doc comment: paths are ALWAYS written with the leading colon in this codebase). Three
///   rows need more than a literal echo:
///   - `Aggregate` contributes TWO probes (`Struct` and `Record`) because its eligibility is
///     runtime-nature-dependent, not fixed per variant.
///   - `Vec` / `wat__std__HashSet` / `wat__std__HashMap` / `Tuple` are recursively
///     atomizable — `is_atomizable` only accepts them via `TypeExpr::Parametric` /
///     `TypeExpr::Tuple` with an atomizable element, never via a bare `Path` — so their
///     probes use a representative atomizable inner type (`:wat::core::i64`).
///   - `wat__core__PersistentMap` / `wat__core__PersistentVector` probe with the SAME
///     `Parametric` shape to prove the checker rejects them regardless (no arm for either
///     head in `is_atomizable`, unlike their `HashMap`/`Vector` siblings).
///   - `RustOpaque`'s `type_name` is a per-instance `&'static str` (`inner.type_path`), not a
///     fixed literal — its probe uses a representative placeholder path, since every
///     `:rust::*` opaque type is uniformly rejected regardless of which one it names.
macro_rules! value_key_eligibility_table {
    (
        $(
            $pat:pat => {
                type_name: $tn:expr,
                key_eligibility: $ke:expr,
                gate: $gate:tt
            }
        ),+ $(,)?
    ) => {
        impl Value {
            /// **TRANSFORMS (clojure-ination):** keyword type-name strings
            pub fn type_name(&self) -> &'static str {
                match self {
                    $( $pat => $tn, )+
                }
            }

            /// The wall (arc 109 BRIEF-key-eligibility-wall.md): whether `self`'s variant may
            /// be used as a hash key. Exhaustive — no `_ =>` wildcard, ever (STOP-1). A new
            /// `Value` variant that skips this classification fails to compile here, exactly
            /// as it already fails to compile in `type_name()`.
            pub fn key_eligibility(&self) -> KeyEligibility {
                match self {
                    $( $pat => $ke, )+
                }
            }

            /// The gate-testable table: for every `Value` variant (and, where eligibility is
            /// runtime-nature-dependent — `Aggregate` — every distinct sub-case), a
            /// checker-facing `TypeExpr` paired with the eligibility `is_atomizable` MUST
            /// agree with. A function, not a `const`/`static` — `TypeExpr::Path` owns a
            /// `String`, which is not const-constructible.
            pub fn all_key_eligibility() -> Vec<(TypeExpr, KeyEligibility)> {
                // `.concat()` over per-row vecs rather than `Vec::new()` + `push` — the
                // init-then-push form is what `clippy::vec_init_then_push` names, and this
                // stone's contract is that it adds no warnings of its own.
                let mut table: Vec<(TypeExpr, KeyEligibility)> = Vec::new();
                $( table.extend(ke_gate_entries!($ke, $gate)); )+
                table
            }
        }
    };
}

value_key_eligibility_table! {
    // ── Hashable: pure data, is_atomizable MUST accept ─────────────────────
    Value::bool(_) => {
        type_name: "wat::core::bool",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::bool".to_string()) ]
    },
    Value::i64(_) => {
        type_name: "wat::core::i64",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::i64".to_string()) ]
    },
    Value::f64(_) => {
        type_name: "wat::core::f64",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::f64".to_string()) ]
    },
    Value::String(_) => {
        type_name: "wat::core::String",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::String".to_string()) ]
    },
    // Arc 220 Stone 220.4 — Vec is recursively atomizable (`is_atomizable`'s
    // `Parametric { head: "wat::core::Vector", .. }` arm); bare-Path is not how the
    // checker admits it, so the probe uses a representative atomizable element.
    Value::Vec(_) => {
        type_name: "wat::core::Vector",
        key_eligibility: KeyEligibility::Hashable,
        gate: [
            TypeExpr::Parametric {
                head: "wat::core::Vector".to_string(),
                args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
            } => KeyEligibility::Hashable
        ]
    },
    // Unit's checker-facing type is `:wat::core::nil` (see `WatAST::NilLit`'s checked type,
    // `src/check.rs:1898`) — NOT `type_name()`'s runtime display label `"()"`. The two are an
    // intentional, pre-existing divergence (display label vs. checker vocabulary), not a
    // classification bug; the probe uses the checker's own string.
    Value::Unit => {
        type_name: "()",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::nil".to_string()) ]
    },
    Value::wat__core__keyword(_) => {
        type_name: "wat::core::keyword",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::keyword".to_string()) ]
    },
    Value::holon__HolonAST(_) => {
        type_name: "wat::holon::HolonAST",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::holon::HolonAST".to_string()) ]
    },
    Value::wat__WatAST(_) => {
        type_name: "wat::WatAST",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::WatAST".to_string()) ]
    },
    // Arc 216 Stone 3 — HashMap recursively atomizable; same Parametric-probe reasoning as Vec.
    Value::wat__std__HashMap(_) => {
        type_name: "wat::core::HashMap",
        key_eligibility: KeyEligibility::Hashable,
        gate: [
            TypeExpr::Parametric {
                head: "wat::core::HashMap".to_string(),
                args: vec![
                    TypeExpr::Path(":wat::core::i64".to_string()),
                    TypeExpr::Path(":wat::core::i64".to_string()),
                ],
            } => KeyEligibility::Hashable
        ]
    },
    // Arc 216 Stone 1 — HashSet recursively atomizable; same Parametric-probe reasoning as Vec.
    Value::wat__std__HashSet(_) => {
        type_name: "wat::core::HashSet",
        key_eligibility: KeyEligibility::Hashable,
        gate: [
            TypeExpr::Parametric {
                head: "wat::core::HashSet".to_string(),
                args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
            } => KeyEligibility::Hashable
        ]
    },
    // Arc 216 Stone 7 — Tuple atomizable iff every element is; is_atomizable admits it via
    // the dedicated `TypeExpr::Tuple` variant, not a bare Path or a Parametric head.
    Value::Tuple(_) => {
        type_name: "wat::core::Tuple",
        key_eligibility: KeyEligibility::Hashable,
        gate: [
            TypeExpr::Tuple(vec![TypeExpr::Path(":wat::core::i64".to_string())])
                => KeyEligibility::Hashable
        ]
    },
    // Arc 293.R2.1 — Aggregate: nature gates BOTH the kind-string and the eligibility.
    // Struct nature → "wat::core::Struct" (NOT atomizable); Record/HolonRecord →
    // "wat::core::Record" (atomizable via the hologram property, arc 234 Stone 234.5).
    // Arc 293 S3-Nature-2 — `Peer` is never the nature of a constructed `AggregateValue`
    // (a peer is a `RustOpaque`, not an aggregate); exhaustiveness only, unreachable at runtime.
    Value::Aggregate(a) => {
        type_name: match a.nature {
            Nature::Struct => "wat::core::Struct",
            Nature::Record | Nature::HolonRecord => "wat::core::Record",
            Nature::Peer => unreachable!("AggregateValue never carries Nature::Peer"),
        },
        key_eligibility: match a.nature {
            Nature::Struct => KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
            Nature::Record | Nature::HolonRecord => KeyEligibility::Hashable,
            Nature::Peer => unreachable!("AggregateValue never carries Nature::Peer"),
        },
        gate: [
            TypeExpr::Path(":wat::core::Struct".to_string())
                => KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
            TypeExpr::Path(":wat::core::Record".to_string()) => KeyEligibility::Hashable,
        ]
    },
    Value::wat__core__Uuid(_) => {
        type_name: "wat::core::Uuid",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::Uuid".to_string()) ]
    },
    // Arc 220 — Stone 242.1 renamed the surface to `char`; this arm was
    // half-propagated (still emitted capital). C1 fixes it.
    Value::wat__core__Char(_) => {
        type_name: "wat::core::char",
        key_eligibility: KeyEligibility::Hashable,
        gate: [ TypeExpr::Path(":wat::core::char".to_string()) ]
    },

    // ── NeverAKey(InteriorMutable): interior-mutable field, proven by reading the ─────
    // ── wrapped struct's own fields (not merely inferred from the Hash arm) ───────────
    // Arc 170 slice 1c — `SenderInner::Comms.closed: AtomicBool` (src/channel/inner.rs).
    // THE finding this stone rests on: the sole current cause of all 18 clippy
    // `mutable_key_type` warnings.
    Value::wat__kernel__Sender(_) => {
        type_name: "wat::kernel::Sender",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::kernel::Sender".to_string()) ]
    },
    // `ChildHandle.reaped: AtomicBool` + `cached_exit: OnceLock<i64>` (src/process/handle.rs)
    // — proven interior mutability, independent of (and not reported by) the Sender chain
    // clippy currently surfaces.
    Value::wat__kernel__ChildHandle(_) => {
        type_name: "wat::kernel::ChildHandle",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::kernel::ChildHandle".to_string()) ]
    },
    // The five `Arc<ThreadOwnedCell<_>>` ML types: `ThreadOwnedCell.cell: UnsafeCell<T>`
    // (src/rust_deps/custodia.rs) is direct, unconditional interior mutability.
    Value::OnlineSubspace(_) => {
        type_name: "wat::holon::OnlineSubspace",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::holon::OnlineSubspace".to_string()) ]
    },
    Value::Reckoner(_) => {
        type_name: "wat::holon::Reckoner",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::holon::Reckoner".to_string()) ]
    },
    Value::Engram(_) => {
        type_name: "wat::holon::Engram",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::holon::Engram".to_string()) ]
    },
    Value::EngramLibrary(_) => {
        type_name: "wat::holon::EngramLibrary",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::holon::EngramLibrary".to_string()) ]
    },
    Value::Hologram(_) => {
        type_name: "wat::holon::Hologram",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::InteriorMutable),
        gate: [ TypeExpr::Path(":wat::holon::Hologram".to_string()) ]
    },

    // ── NeverAKey(OpaqueHandle): pointer-based identity (Arc::ptr_eq); no interior- ───
    // ── mutable field provable from this crate's own struct definitions ───────────────
    Value::wat__core__fn(_) => {
        type_name: "wat::core::fn",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::core::fn".to_string()) ]
    },
    // Arc 170 slice 1c — Receiver is Sender's sibling but carries no AtomicBool of its own
    // (`ReceiverInner::Comms`/`PipeFd` — no interior-mutable field at this level); it is
    // NeverAKey because its `PartialEq` arm is `Arc::ptr_eq` (handle identity), not because
    // of a proven interior-mutable field the way Sender is.
    Value::wat__kernel__Receiver(_) => {
        type_name: "wat::kernel::Receiver",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::kernel::Receiver".to_string()) ]
    },
    // `RustOpaqueInner.payload: Box<dyn Any + Send + Sync>` is erased — opaque by
    // construction, so no interior-mutable field is provable here regardless of what a
    // given shim's payload happens to hold. `type_name` is the per-instance `type_path`;
    // EVERY `:rust::*` path is uniformly rejected by `is_atomizable` (no entry admits any
    // of them), so the gate probes a representative placeholder rather than a real one.
    Value::RustOpaque(inner) => {
        type_name: inner.type_path,
        // `inner` is read above in `type_name`'s match arm but not structurally needed
        // in `key_eligibility`'s — both matches share this one `$pat`, so the binding
        // would otherwise be unused in the latter; `let _ = inner;` reads it without
        // changing behavior (every RustOpaque is uniformly OpaqueHandle regardless of
        // which `:rust::*` type it names).
        key_eligibility: { let _ = inner; KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle) },
        gate: [
            TypeExpr::Path(":wat::rust::__key_eligibility_probe__".to_string())
                => KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle)
        ]
    },
    Value::io__IOReader(_) => {
        type_name: "wat::io::IOReader",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::io::IOReader".to_string()) ]
    },
    Value::io__IOWriter(_) => {
        type_name: "wat::io::IOWriter",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::io::IOWriter".to_string()) ]
    },
    Value::wat__kernel__HandlePool { .. } => {
        type_name: "wat::kernel::HandlePool",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::kernel::HandlePool".to_string()) ]
    },
    // Stone 237.2 — wat__core__clauses: Hash arm hashes the Arc pointer directly (real,
    // not `unreachable!()`) but PartialEq is `Arc::ptr_eq` — identity is pointer-based,
    // not structural, matching OpaqueHandle exactly even though its Hash arm never panics.
    Value::wat__core__clauses(_) => {
        type_name: "wat::core::clauses",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::core::clauses".to_string()) ]
    },
    // Arc 232 Stone 232.1 — extend_def: same real-pointer-hash-not-unreachable shape as
    // wat__core__clauses. (Its `PartialEq` has no explicit arm at all — falls to the
    // cross-variant `_ => false` catch-all, so it is not even reflexively `Arc::ptr_eq`-equal
    // to itself; noted as an adjacent PartialEq-reflexivity observation, not a key-eligibility
    // hazard — it is still uniformly rejected by `is_atomizable` either way.)
    Value::wat__core__extend_def(_) => {
        type_name: "wat::core::extend-def",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::core::extend-def".to_string()) ]
    },
    // Arc 118 — Stream: Hash arm IS `unreachable!()` (undecidable equality on potentially
    // infinite seqs); PartialEq is `Arc::ptr_eq`. Classified OpaqueHandle (identity is
    // pointer-based) rather than InteriorMutable — the exclusion reason is undecidability,
    // not a proven mutable field.
    Value::wat__stream__Stream(_) => {
        type_name: "wat::stream::Stream",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::OpaqueHandle),
        gate: [ TypeExpr::Path(":wat::stream::Stream".to_string()) ]
    },

    // ── NeverAKey(ExcludedByDesign): Hash arm is real/structural, but is_atomizable ────
    // ── does not currently admit it ────────────────────────────────────────────────────
    Value::u8(_) => {
        type_name: "wat::core::u8",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::u8".to_string()) ]
    },
    // is_atomizable has no Parametric arm for either PersistentMap or PersistentVector
    // (unlike HashMap/HashSet/Vector) — rejected regardless of element type.
    Value::wat__core__PersistentMap(_) => {
        type_name: "wat::core::PersistentMap",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [
            TypeExpr::Parametric {
                head: "wat::core::PersistentMap".to_string(),
                args: vec![
                    TypeExpr::Path(":wat::core::i64".to_string()),
                    TypeExpr::Path(":wat::core::i64".to_string()),
                ],
            } => KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign)
        ]
    },
    Value::wat__core__PersistentVector(_) => {
        type_name: "wat::core::PersistentVector",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [
            TypeExpr::Parametric {
                head: "wat::core::PersistentVector".to_string(),
                args: vec![TypeExpr::Path(":wat::core::i64".to_string())],
            } => KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign)
        ]
    },
    Value::Option(_) => {
        type_name: "wat::core::Option",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::Option".to_string()) ]
    },
    Value::Result(_) => {
        type_name: "wat::core::Result",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::Result".to_string()) ]
    },
    Value::Enum(_) => {
        type_name: "wat::core::Enum",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::Enum".to_string()) ]
    },
    // Arc 278 Stone A — foreign dynamic values report their own kind.
    Value::ForeignRecord(_) => {
        type_name: "wat::edn::ForeignRecord",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::edn::ForeignRecord".to_string()) ]
    },
    Value::ForeignVariant(_) => {
        type_name: "wat::edn::ForeignVariant",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::edn::ForeignVariant".to_string()) ]
    },
    Value::Vector(_) => {
        type_name: "wat::holon::Vector",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::holon::Vector".to_string()) ]
    },
    Value::Instant(_) => {
        type_name: "wat::time::Instant",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::time::Instant".to_string()) ]
    },
    Value::Duration(_) => {
        type_name: "wat::time::Duration",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::time::Duration".to_string()) ]
    },
    // Arc 300 stone B — representation-only; not in is_atomizable.
    Value::wat__core__Rational(_) => {
        type_name: "wat::core::rational",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::rational".to_string()) ]
    },
    // Arc 300 stone C1 — full arithmetic type; still not in is_atomizable.
    Value::wat__core__BigInt(_) => {
        type_name: "wat::core::bigint",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::bigint".to_string()) ]
    },
    // Arc 220 Stone 220.4 — List hashes exactly like Vec (hash_sequence, real, recursive)
    // but is deliberately excluded from is_atomizable (own doc comment: "not in
    // is_atomizable") — the EDN-spec cross-type equality with Vec doesn't extend to
    // checker admission.
    Value::wat__core__List(_) => {
        type_name: "wat::core::List",
        key_eligibility: KeyEligibility::NeverAKey(NotAKeyReason::ExcludedByDesign),
        gate: [ TypeExpr::Path(":wat::core::List".to_string()) ]
    },
}

impl Value {
    /// The **declared** type FQDN for this value — the single authority for
    /// "what named type is this value an instance of?" (arc 237 Stone
    /// 237.5.fix-nominal-identity).
    ///
    /// Distinct from `type_name()`, which returns the *variant kind*
    /// (`"wat::core::Enum"`, `"wat::core::Struct"`, …).  Use
    /// `declared_type_name` wherever you need the per-instance declared FQDN
    /// (e.g., `:my::Color`), and `type_name()` only for generic-kind dispatch.
    ///
    /// **Exhaustive match — no bare `_ =>` / `other =>` catch-all for
    /// type-bearing variants.** Every `Value` variant is listed explicitly so
    /// that the Rust compiler rejects a future variant that forgets to supply
    /// its declared-type arm (the exact rot that broke Enum/Newtype).
    ///
    /// Per-form FQDN source:
    /// - `holon__HolonAST` → `extract_classifier` (classifier-wrap FQDN) with
    ///   fallback to `"wat::holon::HolonAST"`.
    /// - `Aggregate(Struct)` → `agg.class` (colon-free FQDN; covers newtype too).
    /// - `Aggregate(Record/HolonRecord)` → `agg.class` (colon-free FQDN).
    /// - `Enum` → `ev.type_path` with leading `:` stripped (the declared enum
    ///   FQDN, e.g. `"my::Color"` — NOT the generic `"wat::core::Enum"`).
    /// - Every primitive/kind-only variant → `self.type_name().to_string()`.
    ///
    /// **TRANSFORMS (clojure-ination):** keyword type-name strings
    pub fn declared_type_name(&self) -> String {
        match self {
            // ── Nominal forms: per-instance declared FQDN ────────────────────
            Value::holon__HolonAST(h) => {
                // rune:solvere(historical-shape) — transitional back-arc into the monolith; extract_classifier lifts to its home at the algebra/ migration stone (docs/arc/2026/06/251-types-as-forms/SCOUT-LIFT-MAP.md); the back-arc resolves then.
                crate::runtime::extract_classifier(h).unwrap_or_else(|| "wat::holon::HolonAST".to_string())
            }
            // Arc 293.R2.1 — Aggregate: class is already colon-free (all natures).
            // Struct nature: type_name was `:my::Point` → stripped → stored as `my::Point`.
            // Record/HolonRecord: class_fqdn was already colon-free.
            Value::Aggregate(a) => a.class.clone(),
            // Enum: type_path is the declared enum FQDN verbatim (e.g.
            // `:my::Color`); strip the leading colon.  Do NOT use
            // self.type_name(), which returns the generic "wat::core::Enum".
            Value::Enum(ev) => ev.type_path.trim_start_matches(':').to_string(),
            // Arc 278 Stone A — foreign dynamic values carry their own declared
            // FQDN self-describingly (colon-free): the record class / the enum class.
            Value::ForeignRecord(fr) => fr.class.clone(),
            Value::ForeignVariant(fv) => fv.enum_class.clone(),

            // ── Primitive / kind-only variants: generic kind string ───────────
            // Listed explicitly (no bare `_ =>`) so the compiler catches any
            // future Value variant that lacks a declared-type arm.
            Value::bool(_) => self.type_name().to_string(),
            Value::i64(_) => self.type_name().to_string(),
            Value::u8(_) => self.type_name().to_string(),
            Value::f64(_) => self.type_name().to_string(),
            Value::String(_) => self.type_name().to_string(),
            Value::Vec(_) => self.type_name().to_string(),
            Value::Unit => self.type_name().to_string(),
            Value::wat__core__keyword(_) => self.type_name().to_string(),
            Value::wat__core__fn(_) => self.type_name().to_string(),
            Value::wat__WatAST(_) => self.type_name().to_string(),
            Value::wat__kernel__Sender(_) => self.type_name().to_string(),
            Value::wat__kernel__Receiver(_) => self.type_name().to_string(),
            Value::wat__std__HashMap(_) => self.type_name().to_string(),
            Value::wat__core__PersistentMap(_) => self.type_name().to_string(),
            Value::wat__core__PersistentVector(_) => self.type_name().to_string(),
            Value::wat__std__HashSet(_) => self.type_name().to_string(),
            Value::RustOpaque(_) => self.type_name().to_string(),
            Value::io__IOReader(_) => self.type_name().to_string(),
            Value::io__IOWriter(_) => self.type_name().to_string(),
            Value::Option(_) => self.type_name().to_string(),
            Value::Result(_) => self.type_name().to_string(),
            Value::Tuple(_) => self.type_name().to_string(),
            Value::wat__kernel__HandlePool { .. } => self.type_name().to_string(),
            Value::wat__kernel__ChildHandle(_) => self.type_name().to_string(),
            Value::Vector(_) => self.type_name().to_string(),
            Value::OnlineSubspace(_) => self.type_name().to_string(),
            Value::Reckoner(_) => self.type_name().to_string(),
            Value::Engram(_) => self.type_name().to_string(),
            Value::EngramLibrary(_) => self.type_name().to_string(),
            Value::Hologram(_) => self.type_name().to_string(),
            Value::Instant(_) => self.type_name().to_string(),
            Value::Duration(_) => self.type_name().to_string(),
            Value::wat__core__Uuid(_) => self.type_name().to_string(),
            Value::wat__core__Char(_) => self.type_name().to_string(),
            Value::wat__core__Rational(_) => self.type_name().to_string(),
            Value::wat__core__BigInt(_) => self.type_name().to_string(),
            Value::wat__core__List(_) => self.type_name().to_string(),
            Value::wat__stream__Stream(_) => self.type_name().to_string(),
            Value::wat__core__clauses(_) => self.type_name().to_string(),
            // Arc 232 Stone 232.1 — registry carrier: generic kind string.
            Value::wat__core__extend_def(_) => self.type_name().to_string(),
        }
    }
}
// Arc 233 Stone 233.2.k: Value::inner(), Value::provenance(), Value::into_tracked() DELETED.
// These helpers were only meaningful while Value::Tracked existed.
// Call sites use TrackedValue::from(value) directly (no need for into_tracked());
// Value is never wrapped post-233.2.k so inner() is a no-op; Value no longer
// carries provenance so provenance() is gone. Use TrackedValue's own .provenance().

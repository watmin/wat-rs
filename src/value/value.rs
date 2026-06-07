//! The `Value` enum and its full cluster — the central runtime data type.
//!
//! Lifted from `src/runtime.rs` (block ~367–1407) in Stone 251.2e.
//! PURE STRUCTURAL MOVE — no behavior change.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crossbeam_channel;
use chrono;
use holon::HolonAST;
use wat_macros::wat_value;

use crate::ast::WatAST;
use crate::fork::ChildHandleInner;
use crate::hologram::Hologram;
use crate::io::{WatReader, WatWriter};
use crate::rust_deps::{RustOpaqueInner, ThreadOwnedCell};
use crate::typed_channel::{SenderInner, ReceiverInner};
use crate::types::TypeExpr;
use crate::value::Function;

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
    /// Transport-polymorphic via [`crate::typed_channel::SenderInner`]:
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
    /// [`crate::typed_channel::ReceiverInner`].
    wat__kernel__Receiver(Arc<ReceiverInner>),
    /// A `:HashMap<K,V>` — Rust std's HashMap natively; stored as
    /// `Arc<HashMap<Value, Value>>` using Stone 216.5a's `impl Hash + PartialEq + Eq
    /// for Value`. No canonical-key crutch; K is the actual HashMap key directly.
    wat__std__HashMap(Arc<HashMap<Value, Value>>),
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
    /// `:wat::kernel::recv` / `try-recv` / `select` and of structural
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
    /// (`make-bounded-channel`, `make-unbounded-channel`, `spawn`,
    /// `select`) and destructured in `let` via the
    /// `((a b ...) rhs)` binder shape. The unit type `:()` stays on
    /// [`Value::Unit`] — tuples start at arity 1.
    Tuple(Arc<Vec<Value>>),
    /// A program's handle — `:ProgramHandle<R>` per FOUNDATION.
    /// Pre-arc-112 the wait mechanism was thread-only (a one-shot
    /// crossbeam result channel); arc 112 lifts the inner repr to
    /// an enum so the SAME wat-level type can carry either an
    /// in-thread receiver (the classic spawn / spawn-program path)
    /// OR a forked-process pid (the fork-program path). Returned
    /// by `:wat::kernel::spawn` (InThread) and stored as the
    /// internal wait field of `:wat::kernel::Process<I,O>` (InThread
    /// or Forked depending on whether the Process came from
    /// spawn-program or fork-program). Consumed by
    /// `:wat::kernel::join` / `:wat::kernel::join-result` (operating
    /// on the bare handle from `spawn` — InThread arm only) and
    /// `:wat::kernel::Process/join-result` (operating on a Process —
    /// dispatches on either variant). The InThread arm preserves
    /// arc 060's catch_unwind contract; the Forked arm wraps
    /// waitpid + exit-code interpretation.
    wat__kernel__ProgramHandle(Arc<ProgramHandleInner>),
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
    /// `:wat::kernel::fork-program-ast` (arc 012 slice 2). Opaque from
    /// wat's POV — produced by fork, consumed by
    /// `:wat::kernel::wait-child`. `Drop` SIGKILLs + reaps if the
    /// caller never waited, keeping zombies out of the process
    /// table.
    wat__kernel__ChildHandle(Arc<ChildHandleInner>),
    /// An instance of a user-declared `:wat::core::struct` type — a
    /// tagged positional tuple. `type_name` carries the struct's
    /// keyword path (e.g., `:wat::holon::CapacityExceeded`); `fields`
    /// holds the values in declaration order. Produced by the
    /// auto-generated `<struct>/new` constructor. Read via the
    /// auto-generated `<struct>/<field>` accessors — both of which are
    /// ordinary [`Function`] entries in the symbol table whose bodies
    /// invoke the `:wat::core::struct-new` / `:wat::core::struct-field`
    /// primitives. No field-by-name dispatch at runtime: accessors are
    /// resolved at parse time like any other keyword-path call.
    Struct(Arc<StructValue>),
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
    /// Arc 234 Stone 234.1 — the holographic dual-form record.
    ///
    /// Carries both projections of an immutable record simultaneously:
    /// - `struct_form` for Rust-fast field access (positional Vec)
    /// - `holon_form` for VSA-aligned operations (HolonAST classifier-wrap)
    ///
    /// Field-type constraints (enforced at macro-expand time by defrecord)
    /// guarantee the two forms are isomorphic. The wat-record IS the hologram.
    ///
    /// `class_fqdn` is the record's class name WITHOUT a leading colon,
    /// e.g. `"myapp::Voltage"`. Identity lives in `holon_form` per Stone
    /// 221.5 canonical bytes seed; Eq and Hash both delegate to it.
    ///
    /// Storage form only — user-facing constructor ships in Stone 234.2
    /// (`:wat::core::defrecord` macro). Polymorphic verbs in Stone 234.3.
    wat__holon__Record {
        /// Record class FQDN — e.g. `"myapp::Voltage"` (no leading colon).
        class_fqdn: Arc<String>,
        /// Ordered field values in declaration order (fast Rust-side access).
        struct_form: Arc<Vec<Value>>,
        /// VSA-aligned dual form: `Bind(Atom(class), Bundle(field-Binds...))`.
        /// Identity lives here; Eq and Hash delegate to this field.
        holon_form: Arc<HolonAST>,
    },
    /// Stone S-C.2c — base (wat) record: the reduced flavor. EDN-restricted data
    /// held in a positional `struct_form`; NO `holon_form`. Field NAMES live on
    /// the class (`RecordDef.field_names`, S-C.2ab); name→index access rides that
    /// path. Structural identity over `(class_fqdn, struct_form)`. Holon-ops are a
    /// teaching error — base has no holon flavor (use a holonic record via
    /// `:wat::holon::Record::def`). Unconstructed at the wat surface until S-C.3
    /// mints `:wat::Record::def` → base.
    wat__Record {
        /// Record class FQDN — e.g. `"my::Pt"` (no leading colon).
        class_fqdn: Arc<String>,
        /// Ordered field values in declaration order (fast Rust-side access).
        /// Structural identity lives here (with `class_fqdn`).
        struct_form: Arc<Vec<Value>>,
    },
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
#[derive(Debug, Clone)]
pub struct Clause {
    /// Parallel vectors: binding names and declared types.
    pub args: Vec<(String, TypeExpr)>,
    /// Stone 241.4 — Optional rest-binder `(name, type)` from `& name <- :T`
    /// in the clause argspec. `None` when no rest-binder is present.
    pub rest_param: Option<(String, TypeExpr)>,
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
}

// ─── end Stone 237.2 structs ──────────────────────────────────────────────────

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
/// `wat__WatAST`, `wat__core__Uuid`, `wat__core__Char`, `wat__holon__Record`,
/// `wat__Record`, `Unit` (`:wat::core::nil`), `Vec` (recursive),
/// `wat__std__HashSet` (recursive), `wat__std__HashMap` (recursive),
/// `Tuple` (iff all element types atomizable).
///
/// **Structurally-equal but NOT atomizable** (natural equality; not predicate-admitted):
/// `u8`, `Option`, `Result`, `Struct`, `Enum`, `Vector` (holon::Vector),
/// `Instant`, `Duration`, `wat__core__List` (not in `is_atomizable`).
///
/// **Opaque handles** (pointer equality; not atomizable; never in HashSet/HashMap keys):
/// `wat__core__fn`, `wat__core__clauses` (pointer-equality like fn),
/// `wat__kernel__Sender`, `wat__kernel__Receiver`,
/// `wat__kernel__ProgramHandle`, `wat__kernel__HandlePool`, `wat__kernel__ChildHandle`,
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
            // --- Structurally-equal but NOT atomizable ---
            (Value::u8(a), Value::u8(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Option(a), Value::Option(b)) => a == b,
            (Value::Result(a), Value::Result(b)) => a == b,
            (Value::Struct(a), Value::Struct(b)) => {
                a.type_name == b.type_name && a.fields == b.fields
            }
            (Value::Enum(a), Value::Enum(b)) => {
                a.type_path == b.type_path
                    && a.variant_name == b.variant_name
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
            (Value::wat__kernel__ProgramHandle(a), Value::wat__kernel__ProgramHandle(b)) => {
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
            // Arc 234 Stone 234.1 — wat__holon__Record: identity lives in holon_form (canonical
            // bytes seed per Stone 221.5). Class_fqdn match checked first for short-circuit
            // performance + structural honesty. struct_form is access optimization; not identity.
            (Value::wat__holon__Record { class_fqdn: a_cls, holon_form: a_h, .. },
             Value::wat__holon__Record { class_fqdn: b_cls, holon_form: b_h, .. }) => {
                a_cls == b_cls && a_h == b_h
            }
            // Stone S-C.2c — wat__Record (base): structural identity over (class_fqdn, struct_form).
            // No holon_form; cross pairs (base vs holonic) fall to `_ => false` below.
            (Value::wat__Record { class_fqdn: a_cls, struct_form: sa },
             Value::wat__Record { class_fqdn: b_cls, struct_form: sb }) => {
                a_cls == b_cls && sa == sb
            }
            // Stone 237.2 — wat__core__clauses: pointer equality (two ClauseSet instances
            // are the same dispatcher iff they are the same Arc). Structural equality
            // over clause bodies is not implemented — same rationale as wat__core__fn.
            (Value::wat__core__clauses(a), Value::wat__core__clauses(b)) => Arc::ptr_eq(a, b),
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
/// `Result`, `Struct`, `Enum`, `Vector`, `Instant`, `Duration`) receive structural
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
            Value::Struct(s) => {
                s.type_name.hash(state);
                s.fields.hash(state);
            }
            Value::Enum(e) => {
                e.type_path.hash(state);
                e.variant_name.hash(state);
                e.fields.hash(state);
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
            Value::wat__kernel__ProgramHandle(_) => unreachable!(
                "Value::wat__kernel__ProgramHandle is not atomizable; is_atomizable predicate in \
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
            // Arc 234 Stone 234.1 — wat__holon__Record: hash delegates to holon_form (canonical form
            // per Stone 221.5). Discriminant tag "wat__holon__Record" prevents cross-variant collisions.
            // struct_form is access optimization; identity lives in holon_form.
            Value::wat__holon__Record { holon_form, .. } => {
                "wat__holon__Record".hash(state);
                holon_form.hash(state);
            }
            // Stone S-C.2c — wat__Record (base): structural hash over (class_fqdn, struct_form).
            // Distinct discriminant tag "wat__Record" prevents cross-variant hash collisions
            // with holonic records (consistent with base-vs-holonic PartialEq returning false).
            Value::wat__Record { class_fqdn, struct_form } => {
                "wat__Record".hash(state);
                class_fqdn.hash(state);
                struct_form.hash(state);
            }
            // Stone 237.2 — wat__core__clauses: hash via Arc pointer (consistent with
            // pointer-equality PartialEq). Same discipline as wat__core__fn.
            Value::wat__core__clauses(cs) => {
                "wat__core__clauses".hash(state);
                (Arc::as_ptr(cs) as usize).hash(state);
            }
        }
    }
}

/// The payload of a [`Value::Struct`] — the struct's fully-qualified
/// declared type name plus its positional field values in declaration
/// order. Cheap to clone (stored in an `Arc` at the Value level).
#[derive(Debug, Clone)]
pub struct StructValue {
    /// Full keyword path of the struct type, e.g.
    /// `:wat::holon::CapacityExceeded`. Matches the declaration's
    /// name verbatim; identity for type-tag comparisons.
    pub type_name: String,
    /// Field values in declaration order. Length matches the
    /// `StructDef::fields` length at construction time; the type
    /// checker enforces alignment.
    pub fields: Vec<Value>,
}

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
    pub fields: Vec<Value>,
}

/// Outcome of a spawned thread's eval, carried on the
/// [`Value::wat__kernel__ProgramHandle`] one-shot channel. Arc 060
/// extends the channel from `Result<Value, EvalBreak>` to this
/// three-state enum so `:wat::kernel::join-result` can discriminate
/// `Panic` (thread eval unwound; `catch_unwind` caught the payload)
/// from `RuntimeErr` (eval returned `Err` normally) at the wat
/// surface. The legacy `:wat::kernel::join` verb still panics the
/// caller on either failure mode (with a `RuntimeError` carrying the
/// captured message), preserving its "I trust this thread" semantic.
// rune:solvere(historical-shape) — SpawnOutcome + ProgramHandleInner are spawn-domain types kept as Value-payload here transitionally; they relocate to the spawn/ home at its migration stone (SCOUT-LIFT-MAP), resolving the asymmetry with fork::ChildHandleInner (imported from its domain module).
#[derive(Debug)]
pub enum SpawnOutcome {
    /// Spawned function returned a Value normally.
    Ok(Value),
    /// Spawned function returned an `Err` from a Result-typed eval
    /// path (or any other RuntimeError surfaced by the eval).
    RuntimeErr(crate::value::RuntimeError),
    /// Spawned function panicked; `catch_unwind` caught the payload.
    /// `message` is always populated (formatted from whatever the
    /// payload carried). `assertion` is `Some(...)` when the panic
    /// was an `AssertionPayload` (arc 016 + 064 — assert-eq's rich
    /// actual/expected/location/frames info); `None` for plain
    /// `panic!()` payloads. Arc 105c preserves arc 064's promise
    /// that assert-eq failures route their structured fields
    /// through run-sandboxed; widening this variant from a bare
    /// String was the minimum substrate change to make that work.
    Panic {
        message: String,
        assertion: Option<crate::assertion::AssertionPayload>,
    },
}

/// Internal repr of a [`Value::wat__kernel__ProgramHandle`] (arc
/// 112 unification). Two variants discriminate the wait mechanism:
///
/// - [`Self::InThread`] — the classic in-thread spawn / spawn-program
///   path. The handle owns one end of a one-shot crossbeam channel
///   the spawned thread sends its [`SpawnOutcome`] on. Wait =
///   `recv` on the channel; produces `ThreadDiedError` variants on
///   failure.
/// - [`Self::Forked`] — the fork-program path. The handle owns an
///   `Arc<ChildHandleInner>` (libc pid + reaped flag + cached exit).
///   Wait = `waitpid` on the pid; produces `ProcessDiedError`
///   variants synthesized from the exit code + (in slice 2b) any
///   captured stderr framing.
///
/// The wat-level type is `:wat::kernel::ProgramHandle<R>` regardless
/// of variant; bare `:wat::kernel::join-result` operates only on the
/// InThread arm and returns `Result<R, ThreadDiedError>`. The new
/// `:wat::kernel::Process/join-result` (arc 112) operates on either
/// arm via the Process struct's internal handle, returning
/// `Result<(), ProcessDiedError>`. The Forked arm of a bare-handle
/// `join-result` call is a usage error today (the handle would have
/// to come from a Process/join field accessor); slice 2a routes
/// the canonical Process wait path through Process/join-result.
#[derive(Debug)]
pub enum ProgramHandleInner {
    InThread(crate::typed_channel::Receiver<SpawnOutcome>),
    Forked(Arc<ChildHandleInner>),
}

impl Value {
    /// **TRANSFORMS (clojure-ination):** keyword type-name strings
    pub fn type_name(&self) -> &'static str {
        match self {
            // Arc 163 slice 3f — flip primitive arms to FQDN.
            // Container arms (Vector/Option/Result/HashMap/HashSet/etc.)
            // stay FQDN (slice 3e shipped them).
            Value::bool(_) => "wat::core::bool",
            Value::i64(_) => "wat::core::i64",
            Value::u8(_) => "wat::core::u8",
            Value::f64(_) => "wat::core::f64",
            Value::String(_) => "wat::core::String",
            Value::Vec(_) => "wat::core::Vector",
            Value::Unit => "()",
            Value::wat__core__keyword(_) => "wat::core::keyword",
            Value::wat__core__fn(_) => "wat::core::fn",
            Value::holon__HolonAST(_) => "wat::holon::HolonAST",
            Value::wat__WatAST(_) => "wat::WatAST",
            // Arc 170 slice 1c — both tier-1 (Crossbeam) and tier-2
            // (PipeFd) backed senders / receivers report the same
            // type_name. The wat-level type checker enforces the
            // tier distinction structurally; runtime type_name names
            // the user-visible kind, not the internal transport.
            Value::wat__kernel__Sender(_) => "wat::kernel::Sender",
            Value::wat__kernel__Receiver(_) => "wat::kernel::Receiver",
            Value::wat__std__HashMap(_) => "wat::core::HashMap",
            Value::wat__std__HashSet(_) => "wat::core::HashSet",
            Value::RustOpaque(inner) => inner.type_path,
            Value::io__IOReader(_) => "wat::io::IOReader",
            Value::io__IOWriter(_) => "wat::io::IOWriter",
            Value::Option(_) => "wat::core::Option",
            Value::Result(_) => "wat::core::Result",
            Value::Tuple(_) => "wat::core::Tuple",
            Value::wat__kernel__ProgramHandle(_) => "wat::kernel::ProgramHandle",
            Value::wat__kernel__HandlePool { .. } => "wat::kernel::HandlePool",
            Value::wat__kernel__ChildHandle(_) => "wat::kernel::ChildHandle",
            Value::Struct(_) => "wat::core::Struct",
            Value::Enum(_) => "wat::core::Enum",
            Value::Vector(_) => "wat::holon::Vector",
            Value::OnlineSubspace(_) => "wat::holon::OnlineSubspace",
            Value::Reckoner(_) => "wat::holon::Reckoner",
            Value::Engram(_) => "wat::holon::Engram",
            Value::EngramLibrary(_) => "wat::holon::EngramLibrary",
            Value::Hologram(_) => "wat::holon::Hologram",
            Value::Instant(_) => "wat::time::Instant",
            Value::Duration(_) => "wat::time::Duration",
            Value::wat__core__Uuid(_) => "wat::core::Uuid",
            // Arc 220
            Value::wat__core__Char(_) => "wat::core::Char",
            // Arc 220 Stone 220.4
            Value::wat__core__List(_) => "wat::core::List",
            // Arc 234 Stone 234.1 — generic kind-string (per-instance FQDN via :wat::core::type).
            // Stone S-C.2c — both flavors share the same static kind-string "wat::Record".
            // Per-instance FQDN is `declared_type_name()` (class_fqdn).
            Value::wat__holon__Record { .. } | Value::wat__Record { .. } => "wat::Record",
            // Stone 237.2 — multi-arity callable dispatcher.
            Value::wat__core__clauses(_) => "wat::core::clauses",
        }
    }

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
    /// - `Struct` → `sv.type_name` with leading `:` stripped (also covers newtype,
    ///   which is a `Value::Struct` at runtime).
    /// - `wat__holon__Record` → `class_fqdn` (already colon-free FQDN).
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
            // Struct: type_name carries the declaration keyword verbatim (e.g.
            // `:my::Point`); strip the leading colon for consistency with the
            // extract_classifier convention.  Newtype is Value::Struct at
            // runtime, so this arm covers it too.
            Value::Struct(sv) => sv.type_name.trim_start_matches(':').to_string(),
            // Record (both flavors): class_fqdn is already colon-free (the defrecord
            // macro stores it without the leading colon). Stone S-C.2c or-pattern.
            Value::wat__holon__Record { class_fqdn, .. }
            | Value::wat__Record { class_fqdn, .. } => class_fqdn.to_string(),
            // Enum: type_path is the declared enum FQDN verbatim (e.g.
            // `:my::Color`); strip the leading colon.  Do NOT use
            // self.type_name(), which returns the generic "wat::core::Enum".
            Value::Enum(ev) => ev.type_path.trim_start_matches(':').to_string(),

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
            Value::wat__std__HashSet(_) => self.type_name().to_string(),
            Value::RustOpaque(_) => self.type_name().to_string(),
            Value::io__IOReader(_) => self.type_name().to_string(),
            Value::io__IOWriter(_) => self.type_name().to_string(),
            Value::Option(_) => self.type_name().to_string(),
            Value::Result(_) => self.type_name().to_string(),
            Value::Tuple(_) => self.type_name().to_string(),
            Value::wat__kernel__ProgramHandle(_) => self.type_name().to_string(),
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
            Value::wat__core__List(_) => self.type_name().to_string(),
            Value::wat__core__clauses(_) => self.type_name().to_string(),
        }
    }
}
// Arc 233 Stone 233.2.k: Value::inner(), Value::provenance(), Value::into_tracked() DELETED.
// These helpers were only meaningful while Value::Tracked existed.
// Call sites use TrackedValue::from(value) directly (no need for into_tracked());
// Value is never wrapped post-233.2.k so inner() is a no-op; Value no longer
// carries provenance so provenance() is gone. Use TrackedValue's own .provenance().

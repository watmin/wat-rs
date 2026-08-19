//! Stream-container registry — the single source of truth for which `Value`s and
//! `TypeExpr`s are sequence containers and what capabilities each one has.
//!
//! # Why this file exists
//!
//! Arc-278 (stone: seq-container-registry). Previously, every sequence op
//! hard-rolled its own per-container `match` arms in `check.rs` AND `runtime.rs`
//! independently. Adding a new container type (e.g. `PersistentVector`) required
//! touching every op on both sides by hand — the O(ops)-per-container cost that
//! caused the drift bugs (arc 220/249/278-0b). This file is the narrow waist:
//!
//! ```text
//!   first  second  third  rest  conj  nth  get  map  filter  fold  ...   ← OPS
//!                 \           \        |       /          /
//!                  ╲───── StreamContainer (capability table) ──────────╱     ← waist
//!                 /          /       |         \           \
//!        Vector  List  PersistentVector  Tuple  WatAstList  HashSet  …    ← TYPES
//! ```
//!
//! An op no longer lists containers; it declares the capability it needs
//! (e.g. `Indexable`) and the accepted set is **derived** from the one
//! capability table, identical on checker side (`of_type`) and runtime side
//! (`of_value`). Adding a new container = one new `enum` variant, whose
//! exhaustiveness ripple forces both classifiers and the capability table to be
//! updated before the code compiles. Drift is unrepresentable.
//!
//! # Scope (strike 1 — positional accessors only)
//!
//! This strike migrates `first`/`second`/`third` (the `Indexable` capability).
//! Other capabilities (`Tail`/`Append`/`Mappable`) are stubbed as methods for
//! later strikes; they do not change observable behavior today.
//!
//! # Capability matrix (current runtime truth; `○ gap` = fillable but not yet)
//!
//! | container          | Indexable | Nth (general positional) | Tail (rest) | Append (conj) | Mappable (map/filter/foldl) | Ordered (reverse/concat) | Measurable | Searchable | Gettable |
//! |--------------------|-----------|---------------------------|-------------|---------------|-----------------------------------|------------------------------------|------------|------------|----------|
//! | Vector             | ✓         | ✓                         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | PersistentVector   | ✓         | ✓                         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | List               | ✓         | ✓                         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | Tuple              | ✓         | ∅ N/A                     | ∅ N/A       | ∅ N/A         | ∅ N/A                            | ∅ N/A                             | ✓          | ✓          | ∅ N/A   |
//! | WatAstList         | ✓         | ✓                         | ✓           | ○ gap         | ○ gap                             | ○ gap                              | ✓          | ✓          | ✓        |
//! | HashSet            | ∅ N/A     | ∅ N/A                     | ∅ N/A       | ✓             | ○ gap                             | ∅ N/A                             | ✓          | ✓          | ✓        |
//! | Stream             | ✓ (idx 0) | ✓                         | ✓           | ∅ N/A         | ○ gap                             | ○ gap                              | ∅ N/A      | ○ gap      | ∅ N/A   |
//!
//! **Nth vs Indexable** (stone 118.B4-0): `Indexable` backs `first`/`second`/`third` (Stream
//! only at index 0 — `first` realizes one cell and rejects any other constant index at
//! compile-time-selected call sites). `Nth` backs `:wat::core::nth`'s *runtime*-supplied index —
//! deliberately a THIRD capability, not a reuse of `Indexable`, because `indexable()` is the one
//! B4-iii is slated to flip to `false` for Stream; sharing it would silently close `nth` on lazy
//! seqs three stones later. `Nth` is `false` for the same two receivers `Indexable` excludes
//! (HashSet — unordered) plus one `Indexable` allows: Tuple (heterogeneous — a *runtime* index
//! cannot be typed per-slot the way a compile-time-constant index can).

use crate::types::TypeExpr;
use crate::value::Value;
use crate::ast::WatAST;

/// The closed set of positional/linear containers.
///
/// Keyed collections (HashMap, PersistentMap, Record) are a separate family
/// — they belong to a sibling `MapContainer` registry (future stone).
///
/// EXHAUSTIVENESS GUARANTEE: adding a new container type = adding a new
/// variant here + filling the `of_type`/`of_value` arms + the capability
/// methods. Any omission is a compile error before the code can ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamContainer {
    Vector,
    List,
    PersistentVector,
    Tuple,
    WatAstList,
    HashSet,
    /// Arc 118 — lazy seq (`Value::wat__stream__Stream`). Supports `first`, `rest`, `empty?`.
    /// NOT indexable by integer (Stream has no O(1) nth); NOT measurable (infinite seqs);
    /// NOT appendable via `conj` (use `cons`); NOT mappable at this strike (HOFs later).
    Stream,
}

impl StreamContainer {
    /// Checker-side classifier: map a reduced `TypeExpr` to the container it
    /// represents, or `None` if it is not a sequence container.
    ///
    /// This is **the only** `TypeExpr → StreamContainer` map. Every checker op
    /// that needs to classify a sequence container must go through here.
    ///
    /// Mirrors the head/path matching already present in
    /// `infer_positional_accessor` (`src/check.rs`) and
    /// `extract_seq_elem` (`src/collection/infer.rs`).
    pub(crate) fn of_type(reduced: &TypeExpr) -> Option<StreamContainer> {
        match reduced {
            // Parametric forms: Vector<T>, List<T>, PersistentVector<T>
            TypeExpr::Parametric { head, .. } if head == "wat::core::Vector" => {
                Some(StreamContainer::Vector)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::List" => {
                Some(StreamContainer::List)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::PersistentVector" => {
                Some(StreamContainer::PersistentVector)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::HashSet" => {
                Some(StreamContainer::HashSet)
            }
            // Arc 118 — Stream<T>: lazy sequence.
            TypeExpr::Parametric { head, .. } if head == "wat::stream::Stream" => {
                Some(StreamContainer::Stream)
            }
            // Bare Path forms: annotations without type parameters
            TypeExpr::Path(p) if p == ":wat::core::Vector" => Some(StreamContainer::Vector),
            TypeExpr::Path(p) if p == ":wat::core::List" => Some(StreamContainer::List),
            TypeExpr::Path(p) if p == ":wat::core::PersistentVector" => {
                Some(StreamContainer::PersistentVector)
            }
            TypeExpr::Path(p) if p == ":wat::WatAST" => Some(StreamContainer::WatAstList),
            TypeExpr::Path(p) if p == ":wat::stream::Stream" => Some(StreamContainer::Stream),
            // Tuple is a structural type, not a named head
            TypeExpr::Tuple(_) => Some(StreamContainer::Tuple),
            // Unresolved type variable, named types, or non-containers
            _ => None,
        }
    }

    /// Runtime-side classifier: map a `Value` to the container it represents,
    /// or `None` if it is not a sequence container.
    ///
    /// This is **the only** `Value → StreamContainer` map for sequence ops.
    /// Every runtime op that needs to classify a sequence container must go
    /// through here.
    ///
    /// Mirrors the arms in `eval_positional_accessor` (`src/runtime.rs`).
    ///
    /// NOTE: `WatAstList` only matches `Value::wat__WatAST` wrapping a
    /// `WatAST::List` form — non-List AST values return `None` (not a
    /// sequence container; the caller handles the TypeMismatch).
    pub(crate) fn of_value(v: &Value) -> Option<StreamContainer> {
        match v {
            Value::Vec(_) => Some(StreamContainer::Vector),
            Value::wat__core__List(_) => Some(StreamContainer::List),
            Value::wat__core__PersistentVector(_) => Some(StreamContainer::PersistentVector),
            Value::Tuple(_) => Some(StreamContainer::Tuple),
            Value::wat__WatAST(ast) => match &**ast {
                WatAST::List(_, _) => Some(StreamContainer::WatAstList),
                // Non-List forms are not sequence containers
                _ => None,
            },
            Value::wat__std__HashSet(_) => Some(StreamContainer::HashSet),
            // Arc 118 — lazy seq.
            Value::wat__stream__Stream(_) => Some(StreamContainer::Stream),
            _ => None,
        }
    }

    // ── Capability table ──────────────────────────────────────────────────────
    //
    // Each method encodes one row of the capability matrix above.
    // `true` = ✓ Supported; `false` = ∅ N/A (the container's NATURE forbids
    // it — not a gap, never to be filled).
    // `○ gap` cells are not represented here yet (they require new runtime
    // arms to be built); they appear in the matrix doc above.

    /// `first`/`second`/`third` — position-indexed element access.
    ///
    /// `true` for ordered containers (Vector, PersistentVector, List, Tuple,
    /// WatAstList). `false` for HashSet (unordered — no canonical "first").
    /// Stone 118.B4-iii — THE WALL: `false` for Stream. A lazy seq yields only through
    /// `:wat::stream::next` now; `first` pretending index=0 is a safe special case hid the
    /// 3x-per-cell force cost of a walk that also calls `empty?`/`rest` (measured: walk C,
    /// no `rest`, still pays 3x). See DESIGN-STONE-118.B4-iii-the-wall.md.
    pub(crate) fn indexable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::Tuple => true,
            StreamContainer::WatAstList => true,
            StreamContainer::HashSet => false,
            // Stone 118.B4-iii — THE WALL: `first` no longer accepts Stream. Use `next`.
            StreamContainer::Stream => false,
        }
    }

    /// `:wat::core::nth` — general positional lookup by a RUNTIME-supplied index.
    ///
    /// Stone 118.B4-0. **Deliberately not `indexable()`**: `first`/`second`/`third` route
    /// through `indexable()`, and B4-iii (a later stone) flips that bit to `false` for
    /// Stream — sharing the gate would silently close `nth` on lazy seqs when that lands.
    /// Also deliberately not `gettable()`: that one is already `false` for Stream, so `nth`
    /// would never reach a lazy seq either way.
    ///
    /// `true` for every ordered, homogeneous container (Vector, PersistentVector, List,
    /// WatAstList — same receivers `nth`'s wat oracle `nth-spec` now type-checks against).
    /// `false` for Tuple (heterogeneous — unlike a compile-time-constant index, a *runtime*
    /// index cannot be typed per-slot) and HashSet (unordered — no positional meaning).
    /// Stone 118.B4-iii — THE WALL: `false` for Stream too (option B, ruled 2026-08-18).
    /// `(nth s i)` on a Vector is O(1); on a Stream it was O(i) via `realize`, walking `i+1`
    /// cells with the SAME syntax — a loop that is linear on one receiver is quadratic on the
    /// other and nothing at the call site says which you hold (measured: n(n+1)/2 forced for
    /// an index-based walk vs n+1 for the equivalent `next`-walk). Use `(drop s i)` then
    /// `next` — the complexity is visible in the spelling.
    pub(crate) fn nth_indexable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::WatAstList => true,
            // Heterogeneous product: a runtime index has no single per-slot type.
            StreamContainer::Tuple => false,
            // Unordered: no positional meaning.
            StreamContainer::HashSet => false,
            // Stone 118.B4-iii — THE WALL: `nth` no longer accepts Stream. Use `(drop s i)` + `next`.
            StreamContainer::Stream => false,
        }
    }

    /// `rest` — return all but the first element.
    ///
    /// Stone 118.B4-iii — THE WALL: `false` for Stream. `rest` on a lazy seq forces one cell
    /// to discard it — the same cost as `next`, but the name hides the force. Use `next` and
    /// keep its `rest` half.
    pub(crate) fn has_tail(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::WatAstList => true,
            // Tuple is heterogeneous: tail changes arity+type → ∅ N/A
            StreamContainer::Tuple => false,
            // HashSet is unordered → ∅ N/A
            StreamContainer::HashSet => false,
            // Stone 118.B4-iii — THE WALL: `rest` no longer accepts Stream. Use `next`.
            StreamContainer::Stream => false,
        }
    }

    /// `conj` — append an element.
    ///
    /// Un-stubbed: strike 2 migrates `conj` classification through this gate.
    /// Arc 118 — Stream: use `cons` instead of `conj` (different operation).
    pub(crate) fn has_append(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::HashSet => true,
            // Tuple is fixed-arity → ∅ N/A
            StreamContainer::Tuple => false,
            // WatAstList: runtime arm not yet built → ○ gap (treated as false until filled)
            StreamContainer::WatAstList => false,
            // Arc 118 — Stream: `conj` is not the Stream idiom; use `cons` explicitly. ∅ N/A.
            StreamContainer::Stream => false,
        }
    }

    /// `map`/`filter`/`foldl` — order-agnostic element transform.
    ///
    /// Un-stubbed: strike 3 migrates HOF classification through this gate.
    /// `true` for Vector and PersistentVector — the only containers the HOF
    /// runtime arms support today (verified against transform.rs eval_vec_map/
    /// filter/foldl). Arc 118.B6b: `foldr` retired — it was `reverse`+`foldl`
    /// wearing a name borrowed from Haskell, where the verb is distinct only
    /// because it is LAZY (a property strict wat cannot have).
    pub(crate) fn mappable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            // Tuple: one fn can't map mixed types → ∅ N/A
            StreamContainer::Tuple => false,
            // WatAstList: ○ gap
            StreamContainer::WatAstList => false,
            // HashSet → set: ○ gap (sensible but not yet built)
            StreamContainer::HashSet => false,
            // Arc 118 — Stream: HOFs (map/filter/etc.) are a later strike. ○ gap.
            StreamContainer::Stream => false,
        }
    }

    /// `reverse`/`concat` — order-dependent sequence ops.
    ///
    /// `true` for ordered, homogeneous, variable-length sequences. `false` for
    /// HashSet (unordered — no defined element order to slice or join) and Tuple
    /// (fixed-arity heterogeneous product). Same nature predicate for both ops.
    ///
    /// ★ Corrected 118.B6b — this header used to also name `take`/`drop`, but they do NOT
    /// consult this gate: 118.2a moved them to `extract_lazyable_elem`'s fixed set
    /// (`collection/infer.rs:1070` records the move: "classification no longer routes through
    /// `ordered()`"). The two live consumers, measured, are `concat` (`collection/eval.rs:763`)
    /// and `reverse` (`collection/transform.rs:51`).
    pub(crate) fn ordered(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::WatAstList => false,
            // ∅ N/A — Tuple fixed-arity heterogeneous; HashSet unordered
            StreamContainer::Tuple => false,
            StreamContainer::HashSet => false,
            // Arc 118 — Stream: reverse/take/drop/concat over lazy seqs are a later strike. ○ gap.
            StreamContainer::Stream => false,
        }
    }

    /// `length` / `empty?` — element count.
    ///
    /// Grounded against the `None =>` arms in `eval_length` and `eval_empty`
    /// (runtime.rs): `Vec`, `PV`, `HashSet`, `List`, `Tuple`, and `WatAstList`
    /// each have inner helpers called there (seq-1b filled).
    pub(crate) fn measurable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::HashSet => true,
            StreamContainer::List => true,
            // seq-1b — filled
            StreamContainer::Tuple => true,
            // seq-1b — filled
            StreamContainer::WatAstList => true,
            // Arc 118 — Stream: length/empty? on an infinite seq would diverge. ∅ N/A for length;
            // `empty?` IS supported via realize (handled directly in eval_empty's Stream arm, not
            // routed through this gate). Keep measurable=false so `length` rejects lazy seqs.
            StreamContainer::Stream => false,
        }
    }

    /// `contains?` — element membership.
    ///
    /// Grounded against the `None =>` arms in `eval_contains` (runtime.rs):
    /// `Vec`, `HashSet`, `PV`, `List`, `Tuple`, and `WatAstList` each have
    /// inner helpers called there (seq-1b filled).
    pub(crate) fn searchable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::HashSet => true,
            // seq-1b — wired
            StreamContainer::List => true,
            // seq-1b — filled
            StreamContainer::Tuple => true,
            // seq-1b — filled
            StreamContainer::WatAstList => true,
            // Arc 118 — Stream: contains? would force the whole (possibly infinite) seq. ○ gap.
            StreamContainer::Stream => false,
        }
    }

    /// `get` — index-based element lookup returning `Option`.
    ///
    /// Grounded against the `None =>` arms in `eval_get` (runtime.rs):
    /// `Vec`, `PV`, `List`, `WatAstList`, and `HashSet` each have inner helpers
    /// called there (seq-1b filled). `Tuple` is a heterogeneous product:
    /// runtime-index cannot be typed (→ `Option<Value>`, lossy) and static
    /// positional access already exists (first/second/third, destructure)
    /// — ∅ N/A, never to be filled.
    pub(crate) fn gettable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            // seq-1b — wired
            StreamContainer::List => true,
            // seq-1b — filled
            StreamContainer::WatAstList => true,
            // seq-1b — membership-as-lookup; filled
            StreamContainer::HashSet => true,
            // ∅ N/A — heterogeneous product; runtime-index can't be typed;
            // use first/second/third or destructure for static access
            StreamContainer::Tuple => false,
            // Arc 118 — Stream: O(1) integer get is not a Stream operation (no random access). ∅ N/A.
            StreamContainer::Stream => false,
        }
    }
}

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
//! | container          | Indexable | Tail (rest) | Append (conj) | Mappable (map/filter/foldl/foldr) | Ordered (reverse/take/drop/concat) | Measurable | Searchable | Gettable |
//! |--------------------|-----------|-------------|---------------|-----------------------------------|------------------------------------|------------|------------|----------|
//! | Vector             | ✓         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | PersistentVector   | ✓         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | List               | ✓         | ✓           | ✓             | ✓                                 | ✓                                  | ✓          | ✓          | ✓        |
//! | Tuple              | ✓         | ∅ N/A       | ∅ N/A         | ∅ N/A                            | ∅ N/A                             | ✓          | ✓          | ∅ N/A   |
//! | WatAstList         | ✓         | ✓           | ○ gap         | ○ gap                             | ○ gap                              | ✓          | ✓          | ✓        |
//! | HashSet            | ∅ N/A     | ∅ N/A       | ✓             | ○ gap                             | ∅ N/A                             | ✓          | ✓          | ✓        |

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
    /// Arc 118 — Stream: `true` for `first` (index=0); index>0 raises at runtime.
    pub(crate) fn indexable(self) -> bool {
        match self {
            StreamContainer::Vector => true,
            StreamContainer::PersistentVector => true,
            StreamContainer::List => true,
            StreamContainer::Tuple => true,
            StreamContainer::WatAstList => true,
            StreamContainer::HashSet => false,
            // Arc 118 — Stream: `first` (index=0) is valid; higher indices raise at runtime.
            StreamContainer::Stream => true,
        }
    }

    /// `rest` — return all but the first element.
    ///
    /// Un-stubbed: strike 2 migrates `rest` classification through this gate.
    /// Arc 118 — Stream: `true` (rest returns the tail, or Empty for an empty seq).
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
            // Arc 118 — Stream: rest returns tail (or Empty) — always valid.
            StreamContainer::Stream => true,
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

    /// `map`/`filter`/`foldl`/`foldr` — order-agnostic element transform.
    ///
    /// Un-stubbed: strike 3 migrates HOF classification through this gate.
    /// `true` for Vector and PersistentVector — the only containers the HOF
    /// runtime arms support today (verified against transform.rs eval_vec_map/
    /// filter/foldl/foldr).
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

    /// `reverse`/`take`/`drop`/`concat` — order-dependent sequence ops.
    ///
    /// `true` for ordered, homogeneous, variable-length sequences. `false` for
    /// HashSet (unordered — no defined element order to slice or join) and Tuple
    /// (fixed-arity heterogeneous product). Same nature predicate for all four ops.
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

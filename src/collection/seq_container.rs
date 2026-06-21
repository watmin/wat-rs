//! Seq-container registry — the single source of truth for which `Value`s and
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
//!                  ╲───── SeqContainer (capability table) ──────────╱     ← waist
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
//! | container          | Indexable | Tail (rest) | Append (conj) | Mappable | Measurable | Searchable | Gettable |
//! |--------------------|-----------|-------------|---------------|----------|------------|------------|----------|
//! | Vector             | ✓         | ✓           | ✓             | ✓        | ✓          | ✓          | ✓        |
//! | PersistentVector   | ✓         | ✓           | ✓             | ✓        | ✓          | ✓          | ✓        |
//! | List               | ✓         | ✓           | ✓             | ○ gap    | ✓          | ✓          | ✓        |
//! | Tuple              | ✓         | ∅ N/A       | ∅ N/A         | ∅ N/A   | ✓          | ✓          | ∅ N/A   |
//! | WatAstList         | ✓         | ✓           | ○ gap         | ○ gap    | ✓          | ✓          | ✓        |
//! | HashSet            | ∅ N/A     | ∅ N/A       | ✓             | ○ gap    | ✓          | ✓          | ✓        |

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
pub(crate) enum SeqContainer {
    Vector,
    List,
    PersistentVector,
    Tuple,
    WatAstList,
    HashSet,
}

impl SeqContainer {
    /// Checker-side classifier: map a reduced `TypeExpr` to the container it
    /// represents, or `None` if it is not a sequence container.
    ///
    /// This is **the only** `TypeExpr → SeqContainer` map. Every checker op
    /// that needs to classify a sequence container must go through here.
    ///
    /// Mirrors the head/path matching already present in
    /// `infer_positional_accessor` (`src/check.rs`) and
    /// `extract_seq_elem` (`src/collection/infer.rs`).
    pub(crate) fn of_type(reduced: &TypeExpr) -> Option<SeqContainer> {
        match reduced {
            // Parametric forms: Vector<T>, List<T>, PersistentVector<T>
            TypeExpr::Parametric { head, .. } if head == "wat::core::Vector" => {
                Some(SeqContainer::Vector)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::List" => {
                Some(SeqContainer::List)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::PersistentVector" => {
                Some(SeqContainer::PersistentVector)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::HashSet" => {
                Some(SeqContainer::HashSet)
            }
            // Bare Path forms: annotations without type parameters
            TypeExpr::Path(p) if p == ":wat::core::Vector" => Some(SeqContainer::Vector),
            TypeExpr::Path(p) if p == ":wat::core::List" => Some(SeqContainer::List),
            TypeExpr::Path(p) if p == ":wat::core::PersistentVector" => {
                Some(SeqContainer::PersistentVector)
            }
            TypeExpr::Path(p) if p == ":wat::WatAST" => Some(SeqContainer::WatAstList),
            // Tuple is a structural type, not a named head
            TypeExpr::Tuple(_) => Some(SeqContainer::Tuple),
            // Unresolved type variable, named types, or non-containers
            _ => None,
        }
    }

    /// Runtime-side classifier: map a `Value` to the container it represents,
    /// or `None` if it is not a sequence container.
    ///
    /// This is **the only** `Value → SeqContainer` map for sequence ops.
    /// Every runtime op that needs to classify a sequence container must go
    /// through here.
    ///
    /// Mirrors the arms in `eval_positional_accessor` (`src/runtime.rs`).
    ///
    /// NOTE: `WatAstList` only matches `Value::wat__WatAST` wrapping a
    /// `WatAST::List` form — non-List AST values return `None` (not a
    /// sequence container; the caller handles the TypeMismatch).
    pub(crate) fn of_value(v: &Value) -> Option<SeqContainer> {
        match v {
            Value::Vec(_) => Some(SeqContainer::Vector),
            Value::wat__core__List(_) => Some(SeqContainer::List),
            Value::wat__core__PersistentVector(_) => Some(SeqContainer::PersistentVector),
            Value::Tuple(_) => Some(SeqContainer::Tuple),
            Value::wat__WatAST(ast) => match &**ast {
                WatAST::List(_, _) => Some(SeqContainer::WatAstList),
                // Non-List forms are not sequence containers
                _ => None,
            },
            Value::wat__std__HashSet(_) => Some(SeqContainer::HashSet),
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
    pub(crate) fn indexable(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            SeqContainer::List => true,
            SeqContainer::Tuple => true,
            SeqContainer::WatAstList => true,
            SeqContainer::HashSet => false,
        }
    }

    /// `rest` — return all but the first element.
    ///
    /// Un-stubbed: strike 2 migrates `rest` classification through this gate.
    pub(crate) fn has_tail(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            SeqContainer::List => true,
            SeqContainer::WatAstList => true,
            // Tuple is heterogeneous: tail changes arity+type → ∅ N/A
            SeqContainer::Tuple => false,
            // HashSet is unordered → ∅ N/A
            SeqContainer::HashSet => false,
        }
    }

    /// `conj` — append an element.
    ///
    /// Un-stubbed: strike 2 migrates `conj` classification through this gate.
    pub(crate) fn has_append(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            SeqContainer::List => true,
            SeqContainer::HashSet => true,
            // Tuple is fixed-arity → ∅ N/A
            SeqContainer::Tuple => false,
            // WatAstList: runtime arm not yet built → ○ gap (treated as false until filled)
            SeqContainer::WatAstList => false,
        }
    }

    /// `map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop`/`concat` — element-wise transformation.
    ///
    /// Un-stubbed: strike 3 migrates HOF classification through this gate.
    /// `true` for Vector and PersistentVector — the only containers the HOF
    /// runtime arms support today (verified against transform.rs eval_vec_map/
    /// filter/foldl/foldr/reverse/take/drop and eval.rs vector_concat_inner).
    pub(crate) fn mappable(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            // List: runtime maps only Vec/PV today → ○ gap (false until filled)
            SeqContainer::List => false,
            // Tuple: one fn can't map mixed types → ∅ N/A
            SeqContainer::Tuple => false,
            // WatAstList: ○ gap
            SeqContainer::WatAstList => false,
            // HashSet → set: ○ gap (sensible but not yet built)
            SeqContainer::HashSet => false,
        }
    }

    /// `length` / `empty?` — element count.
    ///
    /// Grounded against the `None =>` arms in `eval_length` and `eval_empty`
    /// (runtime.rs): `Vec`, `PV`, `HashSet`, `List`, `Tuple`, and `WatAstList`
    /// each have inner helpers called there (seq-1b filled).
    pub(crate) fn measurable(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            SeqContainer::HashSet => true,
            SeqContainer::List => true,
            // seq-1b — filled
            SeqContainer::Tuple => true,
            // seq-1b — filled
            SeqContainer::WatAstList => true,
        }
    }

    /// `contains?` — element membership.
    ///
    /// Grounded against the `None =>` arms in `eval_contains` (runtime.rs):
    /// `Vec`, `HashSet`, `PV`, `List`, `Tuple`, and `WatAstList` each have
    /// inner helpers called there (seq-1b filled).
    pub(crate) fn searchable(self) -> bool {
        match self {
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            SeqContainer::HashSet => true,
            // seq-1b — wired
            SeqContainer::List => true,
            // seq-1b — filled
            SeqContainer::Tuple => true,
            // seq-1b — filled
            SeqContainer::WatAstList => true,
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
            SeqContainer::Vector => true,
            SeqContainer::PersistentVector => true,
            // seq-1b — wired
            SeqContainer::List => true,
            // seq-1b — filled
            SeqContainer::WatAstList => true,
            // seq-1b — membership-as-lookup; filled
            SeqContainer::HashSet => true,
            // ∅ N/A — heterogeneous product; runtime-index can't be typed;
            // use first/second/third or destructure for static access
            SeqContainer::Tuple => false,
        }
    }
}

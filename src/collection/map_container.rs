//! Map-container registry — the single source of truth for which `Value`s and
//! `TypeExpr`s are keyed collections and what capabilities each one has.
//!
//! # Why this file exists
//!
//! Arc-278 (stone: seq-container-registry, strike 5 — MapContainer). Previously,
//! every keyed op hard-rolled its own per-container `match` arms in
//! `collection/infer.rs` AND `runtime.rs` independently. Adding a new keyed
//! primitive (BTreeMap, ordered map, …) required touching every op on both sides
//! by hand — the same O(ops)-per-container drift the `StreamContainer` registry
//! killed for sequences. This file is the **keyed-collection narrow waist**:
//!
//! ```text
//!   assoc  get  contains?  length  empty?  …        ← OPS
//!              \         |          /
//!           MapContainer (capability table)          ← waist
//!              /         |          \
//!     HashMap  PersistentMap  Record  …             ← TYPES
//! ```
//!
//! An op no longer lists containers; it declares the capability it needs and
//! the accepted set is **derived** from the one capability table, identical on
//! checker side (`of_type`) and runtime side (`of_value`). Adding a new keyed
//! container = one new `enum` variant; the exhaustiveness ripple forces both
//! classifiers and the capability table to be updated before the code compiles.
//! Drift is unrepresentable.
//!
//! # Scope (strikes 5 + A — assoc, get, contains?, length, empty?)
//!
//! Strike 5 migrated `assoc` through `can_assoc`. Strike A (collection
//! campaign) routed `get`/`contains?`/`length`/`empty?` through the genuine
//! capability gates (`keyed_lookup`/`has_key`/`measurable`). All four capability
//! methods are now consumed by real op guards in `runtime.rs`; drift is
//! unrepresentable.
//!
//! # Capability matrix (current truth; all gaps filled as of strike A2)
//!
//! | member        | can_assoc | keyed_lookup (get) | has_key | measurable | ordered |
//! |---------------|-----------|--------------------|---------|------------|---------|
//! | HashMap       | ✓         | ✓                  | ✓       | ✓          | ∅       |
//! | PersistentMap | ✓         | ✓                  | ✓       | ✓          | ∅       |
//! | Record        | ✓         | ✓                  | ✓       | ✓          | ✓       |
//!
//! `Record` is ordered (declaration order; `fields` is a `Vec<Value>`).
//! That is a real property with no op consumer yet; promoted to an `ordered()`
//! capability method when keys/vals/seq-over-pairs is built.

use crate::types::{TypeEnv, TypeExpr};
use crate::value::Value;

/// The closed set of keyed (map-like) collection containers.
///
/// Sequence containers (Vector, List, PersistentVector, …) belong to the
/// sibling `StreamContainer` registry in `seq_container.rs`.
///
/// `Record` is a member because it IS a keyed collection on the wire:
/// `edn/render.rs` decodes records as tagged maps; Clojure treats records as
/// maps; `fields: Arc<Vec<Value>>` is the internal repr (ordered field
/// values — declaration order), exactly as `Vec`/`LinkedList`/`VectorSync`
/// are the inner reprs of seq-container members. Inner-repr ≠ family
/// membership; the capability table (below) records its true profile.
///
/// EXHAUSTIVENESS GUARANTEE: adding a new keyed container type = adding a
/// new variant here + filling the `of_type`/`of_value` arms + the capability
/// methods. Any omission is a compile error before the code can ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MapContainer {
    HashMap,
    PersistentMap,
    /// Ordered tagged-map. `fields` is a Vec<Value> of field values in
    /// declaration order. Also ORDERED: a real property with no op consumer yet;
    /// promote to an `ordered()` capability when keys/vals/seq-over-pairs lands.
    Record,
}

impl MapContainer {
    /// Runtime classifier — the ONLY `Value → MapContainer` map. Pure.
    ///
    /// Maps `Value::Aggregate` (Record and HolonRecord natures) to
    /// `MapContainer::Record` (one runtime variant, nature is the only variance — like
    /// `StreamContainer::WatAstList` mapping `WatAST::List` forms).
    pub(crate) fn of_value(v: &Value) -> Option<MapContainer> {
        match v {
            Value::wat__std__HashMap(_) => Some(MapContainer::HashMap),
            Value::wat__core__PersistentMap(_) => Some(MapContainer::PersistentMap),
            Value::Aggregate(a) if a.nature != crate::types::Nature::Struct => {
                Some(MapContainer::Record)
            }
            _ => None,
        }
    }

    /// Checker classifier — the ONLY `TypeExpr → MapContainer` map.
    ///
    /// Takes `&TypeEnv` because `Record` is classified by **subtype** (user
    /// records are subtypes of `:wat::core::Record` / `:wat::holon::Record`), which
    /// requires the type lattice. This diverges from `StreamContainer::of_type`
    /// (which takes only `&TypeExpr`) — the divergence is driven by a real
    /// difference (records have a subtype lattice; seq members are matched by
    /// head/structure), so taking the lattice is honest, not ceremony.
    pub(crate) fn of_type(reduced: &TypeExpr, types: &TypeEnv) -> Option<MapContainer> {
        match reduced {
            TypeExpr::Parametric { head, .. } if head == "wat::core::HashMap" => {
                Some(MapContainer::HashMap)
            }
            TypeExpr::Parametric { head, .. } if head == "wat::core::PersistentMap" => {
                Some(MapContainer::PersistentMap)
            }
            TypeExpr::Path(p)
                if crate::types::is_subtype(p, ":wat::core::Record", types)
                    || crate::types::is_subtype(p, ":wat::holon::Record", types) =>
            {
                Some(MapContainer::Record)
            }
            _ => None,
        }
    }

    // ── Capability table ──────────────────────────────────────────────────────
    //
    // Each method encodes one column of the capability matrix above.
    // `true` = ✓ supported; `false` = ○ gap (fillable, not yet built) or
    // ∅ N/A (the container's nature forbids it — never to be filled).

    /// `assoc` — functional key→value insert/update, returning a new collection.
    ///
    /// `true` for all current members (HashMap, PersistentMap, Record each support
    /// assoc today; the Record arm performs field-update via `record_assoc_inner`).
    pub(crate) fn can_assoc(self) -> bool {
        match self {
            MapContainer::HashMap => true,
            MapContainer::PersistentMap => true,
            MapContainer::Record => true,
        }
    }

    /// `get` — keyed value lookup by key.
    ///
    /// `true` for Record: filled by `record_get_inner` (strike A2).
    pub(crate) fn keyed_lookup(self) -> bool {
        match self {
            MapContainer::HashMap => true,
            MapContainer::PersistentMap => true,
            MapContainer::Record => true, // ✓ filled: record_get_inner (strike A2)
        }
    }

    /// `contains?` / `contains-key?` — membership test by key.
    ///
    /// `true` for Record: filled by `record_contains_field_q_inner` (strike A2).
    pub(crate) fn has_key(self) -> bool {
        match self {
            MapContainer::HashMap => true,
            MapContainer::PersistentMap => true,
            MapContainer::Record => true, // ✓ filled: record_contains_field_q_inner (strike A2)
        }
    }

    /// `length` / `empty?` — element count.
    ///
    /// `true` for Record: filled by `record_length_inner` / `record_empty_q_inner` (strike A2).
    pub(crate) fn measurable(self) -> bool {
        match self {
            MapContainer::HashMap => true,
            MapContainer::PersistentMap => true,
            MapContainer::Record => true, // ✓ filled: record_length_inner / record_empty_q_inner (strike A2)
        }
    }
}

// ── Unit tests — capability table correctness ─────────────────────────────────
//
// All capability methods (`keyed_lookup`, `has_key`, `measurable`, `can_assoc`)
// are now consumed by genuine op gates in `runtime.rs` (the `if m.CAP()` guard
// in `eval_get`, `eval_contains`, `eval_length`, `eval_empty`, and `eval_assoc`
// respectively). These unit tests pin the capability table as OBSERVABLE truth
// so a wrong edit is a test failure before it ships.
//
// `of_value` and `can_assoc` are also exercised end-to-end by the integration
// probe (`tests/probe_map_container.rs`); the integration probe also pins
// `assoc` round-trips and the TypeMismatch error path.
#[cfg(test)]
mod capability_tests {
    use super::MapContainer;

    // ── can_assoc: all three members are assoc-capable today ──────────────────

    #[test]
    fn can_assoc_hashmap() {
        assert!(MapContainer::HashMap.can_assoc());
    }

    #[test]
    fn can_assoc_persistentmap() {
        assert!(MapContainer::PersistentMap.can_assoc());
    }

    #[test]
    fn can_assoc_record() {
        assert!(MapContainer::Record.can_assoc());
    }

    // ── keyed_lookup (get): all three ✓ (Record filled in strike A2) ──────────

    #[test]
    fn keyed_lookup_hashmap() {
        assert!(MapContainer::HashMap.keyed_lookup());
    }

    #[test]
    fn keyed_lookup_persistentmap() {
        assert!(MapContainer::PersistentMap.keyed_lookup());
    }

    #[test]
    fn keyed_lookup_record() {
        assert!(MapContainer::Record.keyed_lookup());
    }

    // ── has_key (contains?): all three ✓ (Record filled in strike A2) ─────────

    #[test]
    fn has_key_hashmap() {
        assert!(MapContainer::HashMap.has_key());
    }

    #[test]
    fn has_key_persistentmap() {
        assert!(MapContainer::PersistentMap.has_key());
    }

    #[test]
    fn has_key_record() {
        assert!(MapContainer::Record.has_key());
    }

    // ── measurable (length/empty?): all three ✓ (Record filled in strike A2) ──

    #[test]
    fn measurable_hashmap() {
        assert!(MapContainer::HashMap.measurable());
    }

    #[test]
    fn measurable_persistentmap() {
        assert!(MapContainer::PersistentMap.measurable());
    }

    #[test]
    fn measurable_record() {
        assert!(MapContainer::Record.measurable());
    }
}

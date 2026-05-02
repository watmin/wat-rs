//! Arc 076 — `Hologram`: therm-routed coordinate-cell store with
//! filtered-argmax readout. HolonAST → HolonAST. Unbounded; entries
//! never evict (the bounded sibling `HologramCache` adapts on top).
//!
//! ## What changed from arc 074
//!
//! Arc 074 took a caller-supplied `pos: f64 ∈ [0, 100]` per put / get.
//! Arc 076 derives the slot from the form's structure: a Thermometer-
//! bearing form routes to `floor(value)` (after normalizing the
//! Thermometer's domain into the store's capacity); every other form
//! routes to slot 0. The caller never passes a coordinate.
//!
//! The filter func is bound at construction (one per store), not per
//! call. `Hologram/get` returns the candidate vec's filtered-argmax —
//! the substrate's unifying lookup primitive.
//!
//! Capacity is `floor(sqrt(d))` for the ambient encoding `d`; same
//! algebra-grid resolution arc 074 used, just expressed at the slot
//! layer instead of the cell layer.
//!
//! ## Slot routing
//!
//! `slot_for_form(ast, capacity)`:
//! - Find the first Thermometer leaf via preorder traversal.
//! - If found: `slot = floor((value - min) / (max - min) * capacity)`,
//!   clamped to `[0, capacity - 1]`.
//! - Otherwise: slot 0.
//!
//! `bracket_slots_for_form(ast, capacity)`:
//! - Therm form: `(floor, ceil)` of the same scaled value, both clamped.
//!   Edge values (where `floor == ceil` or one is out of range) collapse
//!   to a single slot.
//! - Non-therm: `(0, 0)`.
//!
//! ## Filtered-argmax
//!
//! Get gathers candidates from one or two slots (the bracket pair),
//! encodes each candidate's key, computes cosine against the probe,
//! invokes the construction-time filter on each cosine, and returns
//! the highest-cosine candidate that passes the filter — or None.
//!
//! ## Why one filter per store
//!
//! Resolution from arc 076 DESIGN Q1: the user programs the Hologram
//! at construction time. A consumer who wants two filtering modes
//! constructs two stores. Filter at the call site duplicates the
//! decision; binding it once at construction keeps the lookup
//! contract honest.
//!
//! Per `ZERO-MUTEX.md` Tier 2: thread-owned mutable. The wat-side
//! Value variant wraps this in `ThreadOwnedCell` for scope safety
//! with zero Mutex.

use crate::runtime::{apply_function, Function, RuntimeError, SymbolTable, Value};
use crate::span::Span;
use crate::vm_registry::EncoderRegistry;
use holon::{encode, HolonAST, Similarity, Vector};
use std::collections::HashMap;
use std::sync::Arc;

/// Therm-routed coordinate-cell store. Unbounded; entries persist
/// until the store is dropped.
pub struct Hologram {
    /// Slot-indexed storage. Outer length is `capacity` (= floor(sqrt(d))).
    /// Each slot is a HashMap of `(key, val)` pairs whose form's first
    /// Thermometer (if any) lands in that slot. Non-therm forms always
    /// land in slot 0.
    slots: Vec<HashMap<HolonAST, HolonAST>>,
    /// `floor(sqrt(d))` cached at construction; matches the slot count.
    capacity: usize,
    /// The encoding dimension this store was built against. Used to
    /// look up the per-d encoder pair for cosine readout.
    d: usize,
    /// User-supplied filter `:fn(:f64) -> :bool`. Bound at construction,
    /// invoked on every get against each candidate's cosine. The store
    /// returns Some(val) for the highest-cosine candidate that passes.
    filter: Arc<Function>,
}

impl Hologram {
    /// Construct an empty store sized for the given encoding `d`, with
    /// a filter func bound for the lifetime of the store. Capacity is
    /// derived: `floor(sqrt(d))`. d=10000 → 100 slots; d=4096 → 64;
    /// d=1024 → 32.
    pub fn make(d: usize, filter: Arc<Function>) -> Self {
        let capacity = ((d as f64).sqrt().floor() as usize).max(1);
        let slots = (0..capacity).map(|_| HashMap::new()).collect();
        Hologram { slots, capacity, d, filter }
    }

    /// Slot count for this store. Read-only.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The encoding dimension this store was built against. Read-only.
    pub fn dim(&self) -> usize {
        self.d
    }

    /// Total entries across all slots.
    pub fn len(&self) -> usize {
        self.slots.iter().map(|s| s.len()).sum()
    }

    /// `true` iff no slot holds any entry.
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_empty())
    }

    /// Insert `(key, val)`. Slot is derived from `key`'s structure: the
    /// first Thermometer leaf's normalized floor, or slot 0 if no
    /// Thermometer is present. Existing key gets overwritten — the
    /// store is idempotent at the same key.
    pub fn put(&mut self, key: HolonAST, val: HolonAST) {
        let s = slot_for_form(&key, self.capacity);
        self.slots[s].insert(key, val);
    }

    /// Filtered-argmax over the bracket-pair of slots determined by
    /// `probe`'s structure. Therm probes scan two adjacent slots
    /// (floor + ceil); non-therm probes scan slot 0 only. Each
    /// candidate is encoded, cosine-compared against the probe,
    /// filter-tested; the highest-cosine candidate that passes the
    /// filter is returned as `(matched_key, val)`.
    ///
    /// Returns `Ok(Some((key, val)))` on hit, `Ok(None)` on filter-
    /// rejected or empty-bracket. The filter is the construction-time
    /// closure; `sym` and `span` thread through `apply_function` for
    /// the per-candidate filter invocation.
    pub fn find(
        &self,
        probe: &HolonAST,
        sym: &SymbolTable,
        span: Span,
        registry: &EncoderRegistry,
    ) -> Result<Option<(HolonAST, HolonAST)>, RuntimeError> {
        let (left, right) = bracket_slots_for_form(probe, self.capacity);
        let enc = registry.get(self.d);
        let probe_vec: Vector = encode(probe, &enc.vm, &enc.scalar);

        let scan: &[usize] = if left == right { &[left][..] } else { &[left, right][..] };

        let mut best: Option<(f64, HolonAST, HolonAST)> = None;
        for &idx in scan {
            for (k, v) in self.slots[idx].iter() {
                let k_vec = encode(k, &enc.vm, &enc.scalar);
                let cos = Similarity::cosine(&k_vec, &probe_vec);
                let pass = apply_function(
                    Arc::clone(&self.filter),
                    vec![Value::f64(cos)],
                    sym,
                    span.clone(),
                )?;
                let pass_b = match pass {
                    Value::bool(b) => b,
                    other => {
                        // arc 138 slice 3b: span TBD
                        return Err(RuntimeError::MalformedForm {
                            head: ":wat::holon::Hologram/find".into(),
                            reason: format!(
                                "filter returned non-bool: {}",
                                other.type_name()
                            ),
                            span: crate::span::Span::unknown(),
                        })
                    }
                };
                if !pass_b {
                    continue;
                }
                match &best {
                    Some((best_cos, _, _)) if *best_cos >= cos => {}
                    _ => best = Some((cos, k.clone(), v.clone())),
                }
            }
        }
        Ok(best.map(|(_, k, v)| (k, v)))
    }

    /// Convenience: `find` and discard the matched key. Returns just
    /// the value.
    pub fn get(
        &self,
        probe: &HolonAST,
        sym: &SymbolTable,
        span: Span,
        registry: &EncoderRegistry,
    ) -> Result<Option<HolonAST>, RuntimeError> {
        Ok(self.find(probe, sym, span, registry)?.map(|(_, v)| v))
    }

    /// Remove the entry whose key matches `key` exactly. Slot is
    /// derived from the key's structure (same routing as put).
    /// Returns the previously-stored val if the entry existed, else
    /// None. Used by bounded variants (HologramCache) to drop entries
    /// when their LRU sidecar evicts a key.
    pub fn remove(&mut self, key: &HolonAST) -> Option<HolonAST> {
        let s = slot_for_form(key, self.capacity);
        self.slots[s].remove(key)
    }
}

// ─── Slot derivation ─────────────────────────────────────────────

/// First Thermometer leaf in `ast` by preorder traversal; None if the
/// AST contains no Thermometer. Used by `slot_for_form` and
/// `bracket_slots_for_form` to drive the slot routing decision.
///
/// Atom-wrapped HolonAST is traversed transparently. SlotMarker (the
/// arc-073 substrate sentinel) is treated as non-Thermometer for slot
/// routing — it carries no value to extract, so any form containing
/// only SlotMarkers routes to slot 0.
fn find_first_thermometer(ast: &HolonAST) -> Option<(f64, f64, f64)> {
    match ast {
        HolonAST::Thermometer { value, min, max } => Some((*value, *min, *max)),
        HolonAST::Bind(a, b) => {
            find_first_thermometer(a).or_else(|| find_first_thermometer(b))
        }
        HolonAST::Bundle(xs) => xs.iter().find_map(find_first_thermometer),
        HolonAST::Permute(child, _) => find_first_thermometer(child),
        HolonAST::Blend(a, b, _, _) => {
            find_first_thermometer(a).or_else(|| find_first_thermometer(b))
        }
        HolonAST::Atom(inner) => find_first_thermometer(inner),
        // Leaves that don't carry a Thermometer.
        HolonAST::Symbol(_)
        | HolonAST::String(_)
        | HolonAST::I64(_)
        | HolonAST::F64(_)
        | HolonAST::Bool(_)
        | HolonAST::SlotMarker { .. } => None,
    }
}

/// Storage slot for `ast` against a store of `capacity` slots.
/// Therm forms route to `floor((value - min) / (max - min) * capacity)`
/// clamped to `[0, capacity - 1]`. Non-therm forms route to slot 0.
pub fn slot_for_form(ast: &HolonAST, capacity: usize) -> usize {
    match find_first_thermometer(ast) {
        Some((value, min, max)) => scale_to_slot(value, min, max, capacity).0,
        None => 0,
    }
}

/// Bracket pair `(floor_slot, ceil_slot)` for a get probe. Therm
/// probes return both floor and ceil of the scaled value (clamped),
/// so the get scans the candidate vec across the bleed-pair. Non-therm
/// probes return `(0, 0)` — slot 0 alone.
///
/// At domain boundaries (scaled value at slot 0 or `capacity - 1`),
/// floor == ceil; the get scans only one slot.
pub fn bracket_slots_for_form(ast: &HolonAST, capacity: usize) -> (usize, usize) {
    match find_first_thermometer(ast) {
        Some((value, min, max)) => scale_to_slot(value, min, max, capacity),
        None => (0, 0),
    }
}

/// Map a Thermometer's `(value, min, max)` to `(floor_slot, ceil_slot)`
/// for a store of `capacity` slots. Both slots are clamped to
/// `[0, capacity - 1]`. Degenerate `min == max` collapses to slot 0.
fn scale_to_slot(value: f64, min: f64, max: f64, capacity: usize) -> (usize, usize) {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || (max - min).abs() < f64::EPSILON {
        return (0, 0);
    }
    let max_idx = capacity.saturating_sub(1);
    let scaled = ((value - min) / (max - min)) * capacity as f64;
    let floor_ = scaled.floor();
    let ceil_ = scaled.ceil();
    let f = if floor_.is_finite() && floor_ >= 0.0 {
        (floor_ as usize).min(max_idx)
    } else if floor_ < 0.0 {
        0
    } else {
        max_idx
    };
    let c = if ceil_.is_finite() && ceil_ >= 0.0 {
        (ceil_ as usize).min(max_idx)
    } else if ceil_ < 0.0 {
        0
    } else {
        max_idx
    };
    (f, c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc as StdArc;

    fn ast_keyword(name: &str) -> HolonAST {
        HolonAST::keyword(name)
    }

    fn ast_therm(value: f64, min: f64, max: f64) -> HolonAST {
        HolonAST::Thermometer { value, min, max }
    }

    fn bind(a: HolonAST, b: HolonAST) -> HolonAST {
        HolonAST::Bind(StdArc::new(a), StdArc::new(b))
    }

    #[test]
    fn capacity_derived_from_d() {
        // The filter is irrelevant for capacity math; we use a dummy
        // by constructing through the runtime's path in integration
        // tests. Here we exercise the pure helpers.
        let cap = ((10000_f64).sqrt().floor() as usize).max(1);
        assert_eq!(cap, 100);
        let cap = ((4096_f64).sqrt().floor() as usize).max(1);
        assert_eq!(cap, 64);
    }

    #[test]
    fn slot_for_non_therm_is_zero() {
        let cap = 100;
        assert_eq!(slot_for_form(&ast_keyword("alpha"), cap), 0);
        let bound = bind(ast_keyword("name"), ast_keyword("val"));
        assert_eq!(slot_for_form(&bound, cap), 0);
    }

    #[test]
    fn slot_for_canonical_therm() {
        let cap = 100;
        // (therm 42 0 100) → scaled = 42; floor = 42
        assert_eq!(slot_for_form(&ast_therm(42.0, 0.0, 100.0), cap), 42);
        // (therm 0.42 0 100) → scaled = 0.42; floor = 0
        assert_eq!(slot_for_form(&ast_therm(0.42, 0.0, 100.0), cap), 0);
        // (therm 99.42 0 100) → scaled = 99.42; floor = 99
        assert_eq!(slot_for_form(&ast_therm(99.42, 0.0, 100.0), cap), 99);
    }

    #[test]
    fn slot_extracts_from_nested_therm() {
        let cap = 100;
        // Bind(:rsi, Therm(70, 0, 100)) → slot 70
        let form = bind(ast_keyword("rsi"), ast_therm(70.0, 0.0, 100.0));
        assert_eq!(slot_for_form(&form, cap), 70);
    }

    #[test]
    fn bracket_pair_in_interior() {
        let cap = 100;
        // (therm 42.42 0 100) → floor=42, ceil=43
        assert_eq!(
            bracket_slots_for_form(&ast_therm(42.42, 0.0, 100.0), cap),
            (42, 43)
        );
        // (therm 50.0 0 100) → floor=50, ceil=50 (integer; collapses)
        assert_eq!(
            bracket_slots_for_form(&ast_therm(50.0, 0.0, 100.0), cap),
            (50, 50)
        );
    }

    #[test]
    fn bracket_collapses_at_low_edge() {
        let cap = 100;
        // (therm 0.42 0 100) → scaled=0.42; floor=0, ceil=1.
        // BOTH in range — interior bleed-pair (the user's earlier
        // call-out about edges only kicks in when scaled < 1.0 means
        // floor=0 and ceil=1; slot 0 is real, slot 1 is real; this is
        // interior in the impl. The "edge collapses" semantic kicks in
        // only when the scaled value is OUTSIDE [0, capacity], which
        // therm-normalize prevents.)
        assert_eq!(
            bracket_slots_for_form(&ast_therm(0.42, 0.0, 100.0), cap),
            (0, 1)
        );
    }

    #[test]
    fn bracket_clamps_above_capacity() {
        let cap = 100;
        // Out-of-domain therm — value 150 in [0, 100] (a misuse). Maps
        // to scaled=150; floor=150 clamped to 99; ceil=150 clamped to
        // 99. Defensive — therm-normalize would prevent the input, but
        // hand-rolled forms can still produce this.
        assert_eq!(
            bracket_slots_for_form(&ast_therm(150.0, 0.0, 100.0), cap),
            (99, 99)
        );
    }

    #[test]
    fn slot_routing_remaps_asymmetric_domain() {
        // user domain [200, 600]; capacity 100; value 400 → midpoint
        // → slot 50. Confirms the cross-domain mapping is in the slot
        // logic, not in the form construction.
        let cap = 100;
        let form = ast_therm(400.0, 200.0, 600.0);
        assert_eq!(slot_for_form(&form, cap), 50);
        assert_eq!(bracket_slots_for_form(&form, cap), (50, 50));
    }

    #[test]
    fn slot_routing_capacity_is_hologram_property() {
        // Same form, different Hologram capacities → different slots.
        // Capacity is not in the form; it's the Hologram's own
        // resolution. (therm 70 0 100) lands at slot 70 in a 100-slot
        // store and slot 44 in a 64-slot store.
        let form = ast_therm(70.0, 0.0, 100.0);
        assert_eq!(slot_for_form(&form, 100), 70);
        assert_eq!(slot_for_form(&form, 64), 44);
    }
}

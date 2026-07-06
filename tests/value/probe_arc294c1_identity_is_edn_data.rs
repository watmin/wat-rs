//! RED probe — arc 294.c.1: a holon record's identity is its EDN data `(class, fields)`,
//! NOT its derived hologram (flaw #7 — the equality split-brain).
//!
//! THE INVERSION (294 DESIGN flaw #2/#7): `Value::Aggregate`'s Rust `PartialEq`/`Hash`
//! key a HolonRecord's identity on the stored hologram (`HolonForm::Hologram`), while the
//! wat-surface `=` (`values_equal`, runtime.rs) already keys on `(nature, class, fields)`.
//! Two equality contracts on one type, equivalent only by the construction invariant that
//! the hologram is a pure function of the fields. 294 makes EDN the identity (Q-D): the
//! hologram is a DERIVED index, never the identity.
//!
//! This probe exhibits the split-brain directly: two HolonRecords with IDENTICAL
//! `(class, fields)` but DIVERGENT holograms.
//!   - Today (RED): Rust `==` keys on the hologram → the two are UNEQUAL, and their hashes
//!     differ — yet wat-`=` would call them equal. Split-brain.
//!   - After 294.c.1 (GREEN): identity is `(nature, class, fields)` → the two are EQUAL and
//!     hash-equal, regardless of hologram. ONE contract, on the data.
//!
//! Behaviour for normally-constructed records is unchanged: a derived hologram is a pure
//! function of the fields, so same-fields ⟹ same-hologram and the old contract agreed.
//! This probe isolates the ONE case the old contract got wrong — divergent holograms over
//! identical data — which is precisely what flaw #7 names.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::runtime::Value;
use wat::AggregateValue;
use holon::HolonAST;

/// A defrecord-instance-shape hologram: `Bind(Atom(class), Bundle(field-Binds...))`.
/// `marker` lets two holograms over the same logical fields differ structurally
/// (here: the encoded field-value Atom), modelling a hologram that has drifted from
/// — or was simply built differently than — the canonical data.
fn holon_form_with(class: &str, field_name: &str, marker: i64) -> Arc<HolonAST> {
    let bind = HolonAST::Bind(
        Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(field_name))))),
        Arc::new(HolonAST::Atom(Arc::new(HolonAST::I64(marker)))),
    );
    Arc::new(HolonAST::Bind(
        Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(class))))),
        Arc::new(HolonAST::Bundle(Arc::new(vec![bind]))),
    ))
}

fn hash_value(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

/// Two HolonRecords: identical `class` and `fields`, but the holograms differ
/// (marker 1 vs marker 999). The data is the same; only the derived index diverges.
fn divergent_pair() -> (Value, Value) {
    let class = "myapp::Voltage";
    let fields: Arc<Vec<Value>> = Arc::new(vec![Value::i64(42)]);
    let a = Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        fields.clone(),
        holon_form_with(class, "magnitude", 1),
    )));
    let b = Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        fields.clone(),
        holon_form_with(class, "magnitude", 999),
    )));
    (a, b)
}

// ─── The contract: identity is the EDN data, not the hologram ────────────────

#[test]
fn divergent_holograms_over_identical_data_are_equal() {
    let (a, b) = divergent_pair();
    assert_eq!(
        a, b,
        "294.c.1: two HolonRecords with identical (class, fields) must be EQUAL regardless \
         of hologram — identity is the EDN data, not the derived index (flaw #7)"
    );
}

#[test]
fn divergent_holograms_over_identical_data_hash_equal() {
    let (a, b) = divergent_pair();
    assert_eq!(
        hash_value(&a),
        hash_value(&b),
        "294.c.1: equal HolonRecords must hash equal (Hash/Eq consistency) — both must key \
         on (nature, class, fields), never the hologram"
    );
}

// ─── Guard: the flip does NOT erase the real distinctions (regression net) ───

#[test]
fn different_fields_still_unequal() {
    let class = "myapp::Voltage";
    let a = Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        Arc::new(vec![Value::i64(42)]),
        holon_form_with(class, "magnitude", 42),
    )));
    let b = Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        Arc::new(vec![Value::i64(43)]),
        holon_form_with(class, "magnitude", 43),
    )));
    assert_ne!(a, b, "294.c.1: different field values must remain unequal");
}

#[test]
fn different_nature_still_unequal() {
    // Same class + fields, different nature (Record vs HolonRecord) → unequal.
    let class = "myapp::Voltage";
    let fields: Arc<Vec<Value>> = Arc::new(vec![Value::i64(42)]);
    let base = Value::Aggregate(Arc::new(AggregateValue::record(class.to_string(), fields.clone())));
    let holon = Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        fields.clone(),
        holon_form_with(class, "magnitude", 42),
    )));
    assert_ne!(base, holon, "294.c.1: nature is part of identity — Record != HolonRecord");
}

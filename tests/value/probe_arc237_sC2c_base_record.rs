//! FM 2-bis probe — arc 237 Stone S-C.2c: base `Value::wat__core__Record { class_fqdn, struct_form }`.
//!
//! See `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-C2c.md`.
//!
//! S-C.2c mints the BASE record variant beside the holonic
//! `Value::wat__holon__Record { class_fqdn, struct_form, holon_form }`. Base holds the
//! struct flavor ONLY — no `holon_form`; structural identity over `(class_fqdn, struct_form)`;
//! holon-ops are a teaching error (verified by a co-located `runtime.rs` unit test, since
//! `to_holon_inner` is private). Base is UNCONSTRUCTED at the wat surface until S-C.3 (the
//! macro split), so this probe constructs `Value::wat__core__Record` DIRECTLY via the public enum
//! API — the Rust-layer disconfirming probe per the 234.x substrate-probe precedent.
//!
//! COMPILE-RED until the variant exists: this file commits ATOMICALLY with the substrate
//! change that mints the variant (a non-compiling test cannot land on the green baseline
//! alone — `feedback_no_broken_commits`). Its value is the frozen contract sonnet mirrors.
//!
//! Contracts (6 — all PASS post-S-C.2c):
//!   1. structural Eq, equal      — same class + same struct ⇒ `==`
//!   2. structural Eq, class diff  — same struct, diff class ⇒ `!=`
//!   3. structural Eq, struct diff — same class, diff struct ⇒ `!=`
//!   4. base ≠ holonic            — different flavors are different values (guards `_ => false`)
//!   5. Hash consistency          — two equal base records dedup in a HashSet (len 1)
//!   6. type identity             — `type_name() == "wat::core::Record"`; `declared_type_name() == class_fqdn`

use std::collections::HashSet;
use std::sync::Arc;

use holon::HolonAST;
use wat::runtime::Value;
use wat::AggregateValue;

/// A base record `:my::Pt [x y]` with the given two f64 field values.
fn base_pt(x: f64, y: f64) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        "my::Pt".to_string(),
        Arc::new(vec![Value::f64(x), Value::f64(y)]),
    )))
}

/// A base record of an arbitrary class with the given two f64 field values.
fn base_named(class: &str, x: f64, y: f64) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        class.to_string(),
        Arc::new(vec![Value::f64(x), Value::f64(y)]),
    )))
}

/// A HOLONIC record with the "same" logical data — different nature so never equal to base.
fn holonic_pt(x: f64, y: f64) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::holon_record(
        "my::Pt".to_string(),
        Arc::new(vec![Value::f64(x), Value::f64(y)]),
        Arc::new(HolonAST::i64(0)),
    )))
}

// ─── Contract 1: structural Eq — equal ────────────────────────────────────────
#[test]
fn base_eq_equal() {
    assert_eq!(base_pt(1.0, 2.0), base_pt(1.0, 2.0));
}

// ─── Contract 2: structural Eq — class differs ────────────────────────────────
#[test]
fn base_eq_class_differs() {
    assert_ne!(base_named("my::Pt", 1.0, 2.0), base_named("my::Other", 1.0, 2.0));
}

// ─── Contract 3: structural Eq — struct differs ───────────────────────────────
#[test]
fn base_eq_struct_differs() {
    assert_ne!(base_pt(1.0, 2.0), base_pt(1.0, 9.0));
}

// ─── Contract 4: base ≠ holonic (different flavors are different values) ───────
#[test]
fn base_neq_holonic() {
    assert_ne!(base_pt(1.0, 2.0), holonic_pt(1.0, 2.0));
}

// ─── Contract 5: Hash consistency (equal ⇒ same hash; HashSet dedups) ──────────
#[test]
fn base_hash_consistent() {
    let mut set: HashSet<Value> = HashSet::new();
    set.insert(base_pt(1.0, 2.0));
    set.insert(base_pt(1.0, 2.0));
    assert_eq!(set.len(), 1, "two equal base records must dedup to one");
    // a structurally-different base record is a distinct member
    set.insert(base_pt(1.0, 9.0));
    assert_eq!(set.len(), 2);
}

// ─── Contract 6: type identity ────────────────────────────────────────────────
#[test]
fn base_type_identity() {
    let r = base_pt(1.0, 2.0);
    assert_eq!(r.type_name(), "wat::core::Record", "base record's static type name");
    assert_eq!(r.declared_type_name(), "my::Pt", "base record's per-instance class FQDN");
}

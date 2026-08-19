//! Diagnostic probe — `Value::wat__core__Record` variant scaffolding (arc 234 Stone 234.1).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.1 BRIEF. Verifies
//! the substrate scaffolding for the wat-record hologram:
//!
//!   - `Value::wat__core__Record { class_fqdn, struct_form, holon_form }` variant exists
//!   - PartialEq impl keys on (nature, class, fields) — identity is the EDN data,
//!     hologram is a derived index (arc 294.c.1, flaw #7 collapse).
//!     > SUPERSEDED 2026-06-28 by arc 294.c.1: was "delegates to holon_form (per Stone
//!     > 221.5 canonical bytes seed + arc 234 DESIGN equality section)".
//!   - Hash impl keys on (nature, class, fields), consistent with PartialEq (arc 294.c.1).
//!     > SUPERSEDED 2026-06-28 by arc 294.c.1: was "delegates to holon_form".
//!   - Display impl renders `<class_fqdn>(<field_1>, <field_2>, ...)`
//!   - Value::type_name() returns `"wat::core::Record"` (generic kind; Stone 234.1.5 rename)
//!
//! Stone 234.1 is the storage form ONLY. No user-facing constructor in this
//! stone (defrecord macro is Stone 234.2). The probe constructs Value::wat__core__Record
//! directly via Rust API for property verification.
//!
//! `:wat::core::type` dispatch table extension (D6) is verified by:
//!   - Sonnet ships the wat__core__Record arm per the BRIEF (location: src/runtime.rs:14420
//!     TODO marker from Stone 234.0)
//!   - Stone 234.2's defrecord macro tests exercise the dispatch arm end-to-end
//!     when wat-level constructors land
//!
//! Probe contracts (7):
//!   1. Construction — variant literal compiles (variant exists with expected fields)
//!   2. Eq same — two wat__core__Records with same class + same holon_form return true
//!   3. Eq different class — same holon_form structure but different class returns false
//!   4. Eq different fields — same class but different holon_form returns false
//!   5. Hash same — two equal wat__core__Records produce equal hashes (Eq/Hash consistency)
//!   6. Display contains class — format!("{}", record) contains the class_fqdn
//!   7. type_name() generic — returns "wat::core::Record" (Stone 234.1.5 rename; per-instance FQDN
//!      via :wat::core::type instead, verified in Stone 234.2)
//!
//! Initial state: COMPILE-FAILS (no Value::wat__core__Record variant exists).
//! Post-stone: COMPILES + all 7 contracts PASS.
//!
//! Outcomes:
//!   - ALL PASS: Stone 234.1 ships clean; substrate ready for Stone 234.2's
//!     defrecord macro + the rest of arc 234.x
//!   - ANY FAIL: SPECIFIC trap surfaced; Eq/Hash consistency violated OR
//!     Display malformed OR variant shape wrong; resolve before merging

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::runtime::Value;
use wat::{AggregateValue, Nature};
use holon::HolonAST;

/// Build a defrecord-instance-shape HolonAST: Bind(Atom(class), Bundle(field-Binds...))
///
/// Per typed-entities doctrine + arc 227 v3 canonical defrecord shape.
fn make_holon_form(class: &str, fields: Vec<(&str, HolonAST)>) -> Arc<HolonAST> {
    let field_binds: Vec<HolonAST> = fields
        .into_iter()
        .map(|(name, value)| {
            HolonAST::Bind(
                Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(
                    Arc::from(name),
                )))),
                Arc::new(HolonAST::Atom(Arc::new(value))),
            )
        })
        .collect();
    Arc::new(HolonAST::Bind(
        Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(class))))),
        Arc::new(HolonAST::Bundle(Arc::new(field_binds))),
    ))
}

/// Construct a holon_record (Aggregate) fixture for tests.
/// Arc 293.R2.1: wat__holon__Record collapsed to Value::Aggregate(AggregateValue{nature:HolonRecord}).
fn make_record(class: &str, fields: Vec<(&str, Value, HolonAST)>) -> Value {
    // Names come from the SAME `fields` the caller already supplies per-field — not a
    // second, independently hand-typed guess.
    let names: Arc<Vec<String>> = Arc::new(fields.iter().map(|(name, _, _)| name.to_string()).collect());
    let struct_form: Arc<Vec<Value>> = Arc::new(fields.iter().map(|(_, v, _)| v.clone()).collect());
    let holon_field_pairs: Vec<(&str, HolonAST)> = fields
        .iter()
        .map(|(name, _, h)| (*name, h.clone()))
        .collect();
    let holon_form = make_holon_form(class, holon_field_pairs);
    Value::Aggregate(Arc::new(AggregateValue::holon_record(
        class.to_string(),
        names,
        struct_form,
        holon_form,
    )))
}

fn hash_value(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `Value::wat__core__Record { class_fqdn, struct_form, holon_form }` literal
// construction compiles (variant exists with expected fields). This is the
// load-bearing existence check.
#[test]
fn probe_1_variant_construction_compiles() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    // Match destructure proves the variant fields match expected names.
    // Stone S-C.3: Value::wat__core__Record is now BASE (no holon_form);
    // the holonic variant is Value::wat__holon__Record.
    match &r {
        Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
            assert_eq!(a.class.as_ref(), "myapp::Voltage");
            assert_eq!(a.fields.len(), 1);
            assert!(!format!("{:?}", a.holon).is_empty());
        }
        _ => panic!("Probe 1: expected Value::Aggregate(HolonRecord) variant"),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Two wat_records with same class + same holon_form return PartialEq::eq → true.
// Identity is the EDN data (class, fields) (arc 294.c.1); same-class+same-fields ⟹ equal.
// > SUPERSEDED 2026-06-28 by arc 294.c.1: was "Identity lives in the holon_form per
// > Stone 221.5 canonical bytes seed." — identity is now the EDN data, not the hologram.
#[test]
fn probe_2_eq_same_class_same_holon_form() {
    let a = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let b = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(a, b, "Probe 2: two wat__core__Records with same class + holon_form should be equal");
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Two wat_records with DIFFERENT class but same holon_form structure return
// false. Class is part of identity.
#[test]
fn probe_3_eq_different_class() {
    let a = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let b = make_record(
        "myapp::Celsius",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_ne!(a, b, "Probe 3: different class should make records unequal (wat__holon__Record)");
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Two wat_records with same class but different field-values (different
// holon_form contents) return false.
#[test]
fn probe_4_eq_different_field_values() {
    let a = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let b = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(6.0), HolonAST::F64(6.0))],
    );
    assert_ne!(a, b, "Probe 4: different field values should make records unequal (wat__holon__Record)");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// Hash + Eq consistency — two equal wat_records produce equal hashes.
// Required by std::hash::Hash contract (if a == b, then hash(a) == hash(b)).
#[test]
fn probe_5_hash_eq_consistency() {
    let a = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let b = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(a, b, "Probe 5: precondition — wat__core__Records must be equal for hash test");
    assert_eq!(
        hash_value(&a),
        hash_value(&b),
        "Probe 5: equal wat__core__Records must produce equal hashes (Hash+Eq consistency)"
    );
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Debug auto-derive renders the record with its class_fqdn visible. Per
// sub-DESIGN D4: Stone 234.1 ships with Rust's auto-derived Debug only;
// Display impl for the entire Value enum is out-of-scope (would be
// asymmetric with other variants; separate arc).
#[test]
fn probe_6_debug_contains_class() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let rendered = format!("{:?}", r);
    println!("Probe 6 Debug: {}", rendered);
    assert_eq!(
        rendered,
        // Arc 296 G-1 — AggregateValue gained `names` (between `class` and `fields`); the
        // golden was updated to include it, the assertion's intent (Debug renders the
        // variant with its class + fields + hologram visible) is otherwise unchanged.
        r#"Aggregate(AggregateValue { class: "myapp::Voltage", names: ["magnitude"], fields: [f64(5.0)], nature: HolonRecord, holon: Hologram(Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(5.0)))]))) })"#,
        "Probe 6: Debug output must match golden"
    );
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
//
// Value::type_name() returns generic `"wat::core::Record"` (Stone 234.1.5 rename per D2).
// The per-instance FQDN is reachable via `:wat::core::type` which gets the
// wat__core__Record arm in this stone (D6); end-to-end test of the dispatch arm
// lands in Stone 234.2 when defrecord macro provides a wat-level constructor.
#[test]
fn probe_7_type_name_returns_generic_kind() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(
        r.type_name(),
        "wat::core::Record",
        "Probe 7: Value::type_name() should return \"wat::core::Record\" (Stone 234.1.5 rename); per-instance FQDN reachable via :wat::core::type"
    );
}

//! Diagnostic probe — `Value::wat__Record` variant scaffolding (arc 234 Stone 234.1).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.1 BRIEF. Verifies
//! the substrate scaffolding for the wat-record hologram:
//!
//!   - `Value::wat__Record { class_fqdn, struct_form, holon_form }` variant exists
//!   - PartialEq impl delegates to holon_form (per Stone 221.5 canonical bytes
//!     seed + arc 234 DESIGN equality section)
//!   - Hash impl delegates to holon_form (consistent with Eq)
//!   - Display impl renders `<class_fqdn>(<field_1>, <field_2>, ...)`
//!   - Value::type_name() returns `"wat::Record"` (generic kind; Stone 234.1.5 rename)
//!
//! Stone 234.1 is the storage form ONLY. No user-facing constructor in this
//! stone (defrecord macro is Stone 234.2). The probe constructs Value::wat__Record
//! directly via Rust API for property verification.
//!
//! `:wat::core::type` dispatch table extension (D6) is verified by:
//!   - Sonnet ships the wat__Record arm per the BRIEF (location: src/runtime.rs:14420
//!     TODO marker from Stone 234.0)
//!   - Stone 234.2's defrecord macro tests exercise the dispatch arm end-to-end
//!     when wat-level constructors land
//!
//! Probe contracts (7):
//!   1. Construction — variant literal compiles (variant exists with expected fields)
//!   2. Eq same — two wat__Records with same class + same holon_form return true
//!   3. Eq different class — same holon_form structure but different class returns false
//!   4. Eq different fields — same class but different holon_form returns false
//!   5. Hash same — two equal wat__Records produce equal hashes (Eq/Hash consistency)
//!   6. Display contains class — format!("{}", record) contains the class_fqdn
//!   7. type_name() generic — returns "wat::Record" (Stone 234.1.5 rename; per-instance FQDN
//!      via :wat::core::type instead, verified in Stone 234.2)
//!
//! Initial state: COMPILE-FAILS (no Value::wat__Record variant exists).
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

/// Construct a wat__holon__Record fixture for tests.
/// Stone S-C.3 migration: Value::wat__Record is now BASE (no holon_form);
/// Value::wat__holon__Record is HOLONIC (with holon_form). This probe tests
/// the holonic variant (the original Stone 234.1 API target).
fn make_record(class: &str, fields: Vec<(&str, Value, HolonAST)>) -> Value {
    let struct_form: Arc<Vec<Value>> = Arc::new(fields.iter().map(|(_, v, _)| v.clone()).collect());
    let holon_field_pairs: Vec<(&str, HolonAST)> = fields
        .iter()
        .map(|(name, _, h)| (*name, h.clone()))
        .collect();
    let holon_form = make_holon_form(class, holon_field_pairs);
    let class_fqdn = Arc::new(class.to_string());
    Value::wat__holon__Record {
        class_fqdn,
        struct_form,
        holon_form,
    }
}

fn hash_value(v: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `Value::wat__Record { class_fqdn, struct_form, holon_form }` literal
// construction compiles (variant exists with expected fields). This is the
// load-bearing existence check.
#[test]
fn probe_1_variant_construction_compiles() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    // Match destructure proves the variant fields match expected names.
    // Stone S-C.3: Value::wat__Record is now BASE (no holon_form);
    // the holonic variant is Value::wat__holon__Record.
    match &r {
        Value::wat__holon__Record { class_fqdn, struct_form, holon_form } => {
            assert_eq!(class_fqdn.as_str(), "myapp::Voltage");
            assert_eq!(struct_form.len(), 1);
            assert!(!format!("{:?}", holon_form).is_empty());
        }
        _ => panic!("Probe 1: expected Value::wat__holon__Record variant"),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// Two wat_records with same class + same holon_form return PartialEq::eq → true.
// Identity lives in the holon_form per Stone 221.5 canonical bytes seed.
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
    assert_eq!(a, b, "Probe 2: two wat__Records with same class + holon_form should be equal");
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
    assert_eq!(a, b, "Probe 5: precondition — wat__Records must be equal for hash test");
    assert_eq!(
        hash_value(&a),
        hash_value(&b),
        "Probe 5: equal wat__Records must produce equal hashes (Hash+Eq consistency)"
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
    assert!(
        rendered.contains("myapp::Voltage"),
        "Probe 6: Debug output must contain class_fqdn 'myapp::Voltage'; got: {}",
        rendered
    );
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
//
// Value::type_name() returns generic `"wat::Record"` (Stone 234.1.5 rename per D2).
// The per-instance FQDN is reachable via `:wat::core::type` which gets the
// wat__Record arm in this stone (D6); end-to-end test of the dispatch arm
// lands in Stone 234.2 when defrecord macro provides a wat-level constructor.
#[test]
fn probe_7_type_name_returns_generic_kind() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(
        r.type_name(),
        "wat::Record",
        "Probe 7: Value::type_name() should return \"wat::Record\" (Stone 234.1.5 rename); per-instance FQDN reachable via :wat::core::type"
    );
}

//! Diagnostic probe — Stone 234.1.5 variant rename + `:wat::core::Record` namespace promotion.
//!
//! Probes 1-3 and 5 are pure Rust substrate tests using make_record() directly (no WAT startup).
//! Probe 4 uses tests/types/probe_arc234_stone15_namespace_promotion.wat (via startup_beside).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::Value;
use wat::{AggregateValue, Nature};
use holon::HolonAST;

/// Build a defrecord-instance-shape HolonAST: Bind(Atom(class), Bundle(field-Binds...))
fn make_holon_form(class: &str, fields: Vec<(&str, HolonAST)>) -> std::sync::Arc<HolonAST> {
    let field_binds: Vec<HolonAST> = fields
        .into_iter()
        .map(|(name, value)| {
            HolonAST::Bind(
                std::sync::Arc::new(HolonAST::Atom(std::sync::Arc::new(HolonAST::String(std::sync::Arc::from(name))))),
                std::sync::Arc::new(HolonAST::Atom(std::sync::Arc::new(value))),
            )
        })
        .collect();
    std::sync::Arc::new(HolonAST::Bind(
        std::sync::Arc::new(HolonAST::Atom(std::sync::Arc::new(HolonAST::String(std::sync::Arc::from(class))))),
        std::sync::Arc::new(HolonAST::Bundle(std::sync::Arc::new(field_binds))),
    ))
}

/// Construct a holon_record fixture for tests (was wat__holon__Record).
fn make_record(class: &str, fields: Vec<(&str, Value, HolonAST)>) -> Value {
    // Names come from the SAME `fields` the caller already supplies per-field (the same
    // binding the holon-form pairs and struct_form values are built from) — not hand-typed.
    let names: Arc<Vec<String>> = Arc::new(fields.iter().map(|(name, _, _)| name.to_string()).collect());
    let struct_form: Arc<Vec<Value>> =
        Arc::new(fields.iter().map(|(_, v, _)| v.clone()).collect());
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
// `Value::wat__holon__Record { ... }` constructible (variant exists with renamed identifier).
#[test]
fn probe_1_variant_compiles_and_constructs() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    match &r {
        Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
            assert_eq!(a.class.as_ref(), "myapp::Voltage");
            assert_eq!(a.fields.len(), 1);
        }
        _ => panic!("Probe 1: expected Value::Aggregate(HolonRecord) variant"),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// type_name() returns `"wat::core::Record"` — verifies D2 + D5 in lockstep.
#[test]
fn probe_2_type_name_returns_wat_record() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(
        r.type_name(),
        "wat::core::Record",
        "Probe 2: type_name() must return \"wat::core::Record\""
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Eq + Hash consistency — two same-args wat__core__Records are equal AND hash-equal.
#[test]
fn probe_3_eq_hash_consistency_under_rename() {
    let a = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    let b = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(a, b, "Probe 3: two same-args wat__core__Records must compare equal");
    assert_eq!(
        hash_value(&a),
        hash_value(&b),
        "Probe 3: equal wat__core__Records must hash equal"
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `[v <- :wat::core::Record]` annotation type-checks in WAT source.
#[test]
fn probe_4_namespace_type_registration() {
    startup_beside(file!())
        .expect("Probe 4: :wat::core::Record annotation must type-check cleanly");
    // Verify the probe-4 function is callable (returns nil)
    call_beside_value(file!(), ":user::probe-4").expect("Probe 4: eval should succeed");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// class_fqdn extraction returns user-named class; type_name() is separate umbrella.
#[test]
fn probe_5_class_fqdn_extraction_post_rename() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(r.type_name(), "wat::core::Record");
    match &r {
        Value::Aggregate(a) if a.nature == Nature::HolonRecord => {
            assert_eq!(
                a.class.as_ref(),
                "myapp::Voltage",
                "Probe 5: class extraction returns user-named class, NOT umbrella"
            );
        }
        _ => panic!("Probe 5: expected Value::Aggregate(HolonRecord) variant"),
    }
}

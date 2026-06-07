//! Diagnostic probe — Stone 234.1.5 variant rename + `:wat::Record` namespace promotion.
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.1.5 BRIEF. Verifies the
//! corrective stone that renames `Value::wat_record` → `Value::wat__Record` (per
//! arc 109 `__` FQDN convention; matches `Value::wat__core__Uuid`/`wat__core__Char`/
//! `wat__std__HashMap` family) and promotes the record concept-cluster to top-level
//! `:wat::Record::*` namespace (peer of `:wat::holon::*`/`:wat::kernel::*`/etc.).
//!
//! Stone 234.1.5 is the foundation that every subsequent arc 234 stone operates on.
//! NO new primitives. NO new behavior. Pure rename + type registration.
//!
//! Probe contracts (5):
//!   1. Variant compile-pass — `Value::wat__Record { ... }` constructible via Rust helper
//!   2. type_name() returns `"wat::Record"` (was `"wat::core::wat_record"`)
//!   3. Eq + Hash consistency under rename — equal records hash equal
//!   4. Type registration accepts `[v <- :wat::Record]` — wat source type-checks
//!   5. class_fqdn extraction returns user-named class (eval_type integration preserved)
//!
//! Initial state: 5/5 FAIL with compile-error on `Value::wat__Record` OR type-check error
//! on `[v <- :wat::Record]`.
//!
//! Post-stone: 5/5 PASS.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use holon::HolonAST;

/// Build a defrecord-instance-shape HolonAST: Bind(Atom(class), Bundle(field-Binds...))
/// Mirrors Stone 234.1's probe helper.
fn make_holon_form(class: &str, fields: Vec<(&str, HolonAST)>) -> Arc<HolonAST> {
    let field_binds: Vec<HolonAST> = fields
        .into_iter()
        .map(|(name, value)| {
            HolonAST::Bind(
                Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(name))))),
                Arc::new(HolonAST::Atom(Arc::new(value))),
            )
        })
        .collect();
    Arc::new(HolonAST::Bind(
        Arc::new(HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(class))))),
        Arc::new(HolonAST::Bundle(Arc::new(field_binds))),
    ))
}

/// Construct a wat__holon__Record fixture for tests. (Stone S-C.3: holonic variant.)
/// Stone S-C.3 migration: Value::wat__Record is now BASE (no holon_form);
/// Value::wat__holon__Record is HOLONIC (carries holon_form for holon-ops).
fn make_record(class: &str, fields: Vec<(&str, Value, HolonAST)>) -> Value {
    let struct_form: Arc<Vec<Value>> =
        Arc::new(fields.iter().map(|(_, v, _)| v.clone()).collect());
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

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// Variant compile-pass — `Value::wat__Record { class_fqdn, struct_form, holon_form }`
// constructible (variant exists with renamed identifier).
#[test]
fn probe_1_variant_compiles_and_constructs() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    match &r {
        Value::wat__holon__Record { class_fqdn, struct_form, .. } => {
            assert_eq!(class_fqdn.as_str(), "myapp::Voltage");
            assert_eq!(struct_form.len(), 1);
        }
        _ => panic!("Probe 1: expected Value::wat__holon__Record variant"),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// type_name() returns `"wat::Record"` — verifies D2 + D5 in lockstep. The renamed
// variant's type_name must match the umbrella FQDN registered in check.rs.
#[test]
fn probe_2_type_name_returns_wat_record() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );
    assert_eq!(
        r.type_name(),
        "wat::Record",
        "Probe 2: type_name() must return \"wat::record\" (NOT old \"wat::core::wat_record\")"
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// Eq + Hash consistency under rename — two same-args wat__Records are equal AND
// hash-equal (Stone 221.5 canonical-bytes invariants survive the rename).
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

    assert_eq!(a, b, "Probe 3: two same-args wat__Records must compare equal");
    assert_eq!(
        hash_value(&a),
        hash_value(&b),
        "Probe 3: equal wat__Records must hash equal (Hash/Eq consistency invariant)"
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// Type registration accepts `[v <- :wat::Record]` — declaring this type
// annotation in a wat source must parse + type-check cleanly. Verifies the check.rs
// TypeDef registration shipped in this stone.
#[test]
fn probe_4_namespace_type_registration() {
    let src = r#"
(:wat::core::defn :user::accept-record [_v <- :wat::Record] -> :wat::core::nil nil)

(:wat::core::defn :user::compute [] -> :wat::core::nil nil)
"#;
    match run_compute(src) {
        Ok(_) => {} // PASS — startup_from_source accepted the type annotation
        Err(e) => panic!(
            "Probe 4: :wat::Record annotation must type-check cleanly; got: {}",
            e
        ),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// class_fqdn extraction returns user-named class — Stone 234.0's eval_type arm
// behavior preserved through the rename (D3). The umbrella type_name() = "wat::Record"
// is separate from the per-instance class_fqdn = "myapp::Voltage".
#[test]
fn probe_5_class_fqdn_extraction_post_rename() {
    let r = make_record(
        "myapp::Voltage",
        vec![("magnitude", Value::f64(5.0), HolonAST::F64(5.0))],
    );

    // type_name() returns the umbrella ("wat::Record")
    assert_eq!(r.type_name(), "wat::Record");

    // class_fqdn field carries the user-named class ("myapp::Voltage").
    // make_record() builds a Value::wat__holon__Record (holonic — carries
    // holon_form for holon-ops), which is the variant produced by the
    // defrecord path post arc 237 S-C.2c base/holonic split.
    match &r {
        Value::wat__holon__Record { class_fqdn, .. } => {
            assert_eq!(
                class_fqdn.as_str(),
                "myapp::Voltage",
                "Probe 5: class_fqdn extraction returns user-named class, NOT umbrella"
            );
        }
        _ => panic!("Probe 5: expected Value::wat__holon__Record variant"),
    }
}

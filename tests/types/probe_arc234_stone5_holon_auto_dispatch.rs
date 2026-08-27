//! Diagnostic probe — `:wat::holon::*` auto-dispatch on `Value::wat__core__Record`
//! (arc 234 Stone 234.5).
//!
//! Wat source: tests/types/probe_arc234_stone5_holon_auto_dispatch.wat (loaded via startup_beside).

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

fn run(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(StartupError::from)
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::to-holon r)` returns the record's holon_form unchanged.
#[test]
fn probe_1_to_holon_returns_holon_form() {
    match run(":user::probe-1") {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert_eq!(s, r#"Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(5.0)))]))"#);
        }
        Ok(other) => panic!("Probe 1: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::cosine r1 r2)` on two identical-class records returns
// `:wat::holon::CosineOutcome` (arc 278 the cosine outcome wall — cosine is
// no longer a bare f64). Two identical, non-zero-magnitude records can never
// hit Degenerate/DimensionMismatch, so this asserts the Similarity variant.
#[test]
fn probe_2_cosine_accepts_records() {
    match run(":user::probe-2") {
        Ok(Value::Enum(ev)) => {
            assert_eq!(
                ev.type_path, ":wat::holon::CosineOutcome",
                "Probe 2: expected CosineOutcome; got type_path {:?}",
                ev.type_path
            );
            match (ev.variant_name.as_str(), ev.fields.as_slice()) {
                ("Similarity", [Value::f64(f)]) => {
                    assert!(f.is_finite(), "Probe 2: cosine should return finite f64; got {}", f);
                    assert!(
                        (-1.0..=1.0).contains(f),
                        "Probe 2: cosine should be in [-1, 1]; got {}",
                        f
                    );
                }
                other => panic!("Probe 2: expected CosineOutcome::Similarity[f64]; got {:?}", other),
            }
        }
        Ok(other) => panic!("Probe 2: expected Value::Enum(CosineOutcome); got {:?}", other),
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::Bind classifier-h r)` with record as right arg.
#[test]
fn probe_3_bind_accepts_record_as_right() {
    match run(":user::probe-3") {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert_eq!(s, r#"Bind(Atom(String("wrapper")), Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(5.0)))])))"#);
        }
        Ok(other) => panic!("Probe 3: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::Bundle [r1 r2 r3])` accepts records as children.
#[test]
fn probe_4_bundle_accepts_records_as_children() {
    match run(":user::probe-4") {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert_eq!(s, r#"Bundle([Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(1.0)))])), Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(2.0)))])), Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(3.0)))]))])"#);
        }
        Ok(other) => panic!("Probe 4: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::extract-classifier r)` returns the record's class_fqdn as String.
#[test]
fn probe_5_extract_classifier_on_record() {
    match run(":user::probe-5") {
        Ok(Value::String(s)) => assert_eq!(
            s.as_str(),
            "myapp::Voltage",
            "Probe 5: extract-classifier should return class_fqdn 'myapp::Voltage'"
        ),
        Ok(other) => panic!("Probe 5: expected Value::String; got {:?}", other),
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// Mixed records + raw HolonASTs through Bind + Bundle composition.
#[test]
fn probe_6_mixed_records_and_holon_asts() {
    match run(":user::probe-6") {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert_eq!(s, r#"Bind(Atom(String("wrapper")), Bundle([Bind(Atom(String("myapp::Voltage")), Bundle([Bind(Atom(String("magnitude")), Atom(F64(5.0)))])), Atom(String("marker"))]))"#);
        }
        Ok(other) => panic!("Probe 6: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

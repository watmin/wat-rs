//! Diagnostic probe — `:wat::holon::*` auto-dispatch on `Value::wat__core__Record`
//! (arc 234 Stone 234.5).
//!
//! Wat source: tests/types/probe_arc234_stone5_holon_auto_dispatch.wat (loaded via startup_beside).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!(&format!("({fn_name})")).map_err(|e| format!("parse: {:?}", e))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::to-holon r)` returns the record's holon_form unchanged.
#[test]
fn probe_1_to_holon_returns_holon_form() {
    match run(":user::probe-1") {
        Ok(Value::holon__HolonAST(h)) => {
            let s = format!("{:?}", h);
            assert!(
                s.contains("myapp::Voltage") || s.contains("Voltage"),
                "Probe 1: holon_form should mention the class; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 1: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `(:wat::holon::cosine r1 r2)` on two identical-class records returns f64.
#[test]
fn probe_2_cosine_accepts_records() {
    match run(":user::probe-2") {
        Ok(Value::f64(f)) => {
            assert!(f.is_finite(), "Probe 2: cosine should return finite f64; got {}", f);
            assert!(
                (-1.0..=1.0).contains(&f),
                "Probe 2: cosine should be in [-1, 1]; got {}",
                f
            );
        }
        Ok(other) => panic!("Probe 2: expected Value::f64; got {:?}", other),
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
            assert!(
                s.contains("Bind") || s.contains("wrapper"),
                "Probe 3: result should be a Bind containing the wrapper; got {}",
                s
            );
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
            assert!(s.contains("Bundle"), "Probe 4: result should be a Bundle; got {}", s);
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
            assert!(
                s.contains("Bind") && s.contains("Bundle"),
                "Probe 6: result should contain Bind + Bundle composition; got {}",
                s
            );
        }
        Ok(other) => panic!("Probe 6: expected Value::holon__HolonAST; got {:?}", other),
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

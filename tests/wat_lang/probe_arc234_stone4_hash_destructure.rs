//! Diagnostic probe — let-binding hash-destructure (arc 234 Stone 234.4).
//!
//! Verifies the Clojure-style `{var :field var2 :field2 ...}` brace-form
//! in let binding position. Receiver-polymorphic over record/struct/HashMap.
//!
//! Match-arm hash-destructure is named follow-up Stone 234.4.match
//! (NOT 234.4 scope; will ship separately).
//!
//! Probe contracts (6):
//!   1. Single-field record destructure
//!   2. Multi-field record destructure (3 fields)
//!   3. HashMap destructure with present key (Some)
//!   4. HashMap destructure with missing key (None)
//!   5. UnknownField error on record bad field
//!   6. Multiple bindings in same let (two destructures)
//!
//! Initial state: 6/6 FAIL with parser/check errors.
//! Post-stone: 6/6 PASS.

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_single_field_record_destructure() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe1-single-field)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::f64(f) => assert!((f - 5.0).abs() < 1e-9, "got {}", f),
        other => panic!("Probe 1: expected f64; got {:?}", other),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_multi_field_record_destructure() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe2-multi-field)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::String(s) => assert_eq!(s.as_str(), "hello"),
        other => panic!("Probe 2: expected String; got {:?}", other),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_3_hashmap_destructure_some() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe3-hashmap-some)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::i64(n) => assert_eq!(n, 8080),
        other => panic!("Probe 3: expected i64; got {:?}", other),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_hashmap_destructure_none() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe4-hashmap-none)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::bool(b) => assert!(b, "Probe 4: expected true (None branch)"),
        other => panic!("Probe 4: expected bool; got {:?}", other),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn probe_5_unknown_field_errors() {
    // The unknown-field error fires at eval time (checker permits the form, runtime rejects).
    let world =
        startup_from_file("tests/wat_lang/probe_arc234_stone4_hash_destructure.wat.bad")
            .expect("startup should succeed; error fires at eval time");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(v) => panic!("Probe 5 FAILED: expected error; got Ok({:?})", v.value_owned()),
        Err(e) => {
            let msg = format!("{:?}", e);
            assert_eq!(
                msg,
                r#"RuntimeError { span: Span { file: "tests/wat_lang/probe_arc234_stone4_hash_destructure.wat.bad", line: 8, col: 25, end_line: 8, end_col: 46 }, kind: UnknownField { record_class: "myapp::Voltage", field: "nonexistent", available: ["magnitude"] } }"#,
                "Probe 5: expected exact UnknownField error"
            );
        }
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_6_multiple_destructures_in_same_let() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe6-multiple)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::f64(f) => assert!((f - 10.5).abs() < 1e-9, "got {}", f),
        other => panic!("Probe 6: expected f64; got {:?}", other),
    }
}

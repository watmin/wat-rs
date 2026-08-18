//! The compiled program on disk deduces that we are the datamancer.
//!
//! `datamancer.rete.edn` is the residual. This fixture has types and the ask.
//! It has no `defrule` and no `compile-all`. Rust reads the log; wat evaluates it.
//! The same program distinguishes the practitioner from the impostor.

use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn fixture_wat() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/rete/probe_arc278_rete_edn.wat");
    std::fs::read_to_string(&path).expect("disk-program fixture")
}

fn datamancer_edn_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/rete/datamancer.rete.edn")
}

fn edn_from_disk() -> String {
    std::fs::read_to_string(datamancer_edn_path()).expect("datamancer.rete.edn must exist on disk")
}

fn call_with_edn(fn_name: &str, txt: &str) -> Value {
    let world = startup_beside(file!()).expect("freeze disk-program fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name}"))
        .clone();
    apply_function(
        func,
        vec![Value::String(Arc::new(txt.to_string()))],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .unwrap_or_else(|e| panic!("{fn_name}: {e}"))
}

fn pair_i64(v: Value) -> (i64, i64) {
    match v {
        Value::wat__core__PersistentVector(pv) => {
            let a = match pv.get(0) {
                Some(Value::i64(x)) => *x,
                other => panic!("first missing: {other:?}"),
            };
            let b = match pv.get(1) {
                Some(Value::i64(x)) => *x,
                other => panic!("second missing: {other:?}"),
            };
            (a, b)
        }
        other => panic!("expected [who hollow], got {other:?}"),
    }
}

#[test]
fn fixture_does_not_compile_the_program() {
    let src = fixture_wat();
    assert!(
        !src.contains(":wat::rete::compile-all"),
        "disk-program fixture must not compile-all — the program is datamancer.rete.edn"
    );
    assert!(
        !src.contains(":wat::rete::defrule"),
        "disk-program fixture must not carry the rule source"
    );
}

#[test]
fn practice_on_disk_program_deduces_datamancer() {
    let txt = edn_from_disk();
    assert!(
        txt.contains("#wat.rete/Export"),
        "datamancer.rete.edn must be the compiled program (one tag)"
    );
    assert!(
        !txt.contains("Symbol(Identifier"),
        "Export must not ship Debug of WatAST — rbind is the slot name, not a span"
    );
    let (who, hollow) = pair_i64(call_with_edn(":user::practice", &txt));
    assert_eq!(who, 1, "the practice must deduce Datamancer");
    assert_eq!(hollow, 0, "the practitioner is not hollow");
    let sigil = match call_with_edn(":user::sigil", &txt) {
        Value::String(s) => (*s).clone(),
        other => panic!("expected sigil String, got {other:?}"),
    };
    assert_eq!(sigil, "RESIDVVM EST PROGRAMMA");
}

#[test]
fn impostor_on_disk_program_is_hollow() {
    let txt = edn_from_disk();
    let (who, hollow) = pair_i64(call_with_edn(":user::impostor", &txt));
    assert_eq!(who, 0, "a gap without reading the log is not the datamancer");
    assert_eq!(hollow, 1, "the summary talking in our voice is Hollow");
}

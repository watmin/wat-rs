//! FM 2-bis probe — arc 237 Stone S-B.1: `:wat::core::recordtype` + `TypeDef::Record`.
//!
//! Wat source: tests/types/probe_arc237_sB1_recordtype.wat (loaded via startup_beside).
//! Probe 06 (negative) uses tests/types/probe_arc237_sB1_recordtype.wat.bad.

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_bool(fn_name: &str) -> Result<bool, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!(&format!("({fn_name})")).map_err(|e| format!("parse: {:?}", e))?;
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))?
    {
        Value::bool(b) => Ok(b),
        other => Err(format!("expected bool; got {:?}", other)),
    }
}

fn assert_true(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(true) => {}
        other => panic!("expected true for `{}`; got {:?}", fn_name, other),
    }
}

fn assert_false(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(false) => {}
        other => panic!("expected false for `{}`; got {:?}", fn_name, other),
    }
}

// ─── Probe 01: recordtype form registers ────────────────────────────────────
#[test]
fn probe_01_recordtype_registers() {
    assert_true(":user::probe-01");
}

// ─── Probe 02: is-X? synthesized ∀T — THE asymmetry-killer ─────────────────
#[test]
fn probe_02_is_predicate_synthesized_forall_t() {
    assert_false(":user::probe-02");
}

// ─── Probe 03: edge wired by recordtype (Circle is-a Record) ────────────────
#[test]
fn probe_03_edge_wired() {
    assert_true(":user::probe-03");
}

// ─── Probe 04: directional (Record is NOT-a Circle) ─────────────────────────
#[test]
fn probe_04_directional() {
    assert_false(":user::probe-04");
}

// ─── Probe 05: holon-flavor parent + transitive (Sphere→holon::Record→Record) ─
#[test]
fn probe_05_holon_flavor_transitive() {
    assert_true(":user::probe-05a");
    assert_true(":user::probe-05b");
}

// ─── Probe 06: unknown parent rejected at registration ──────────────────────
#[test]
fn probe_06_unknown_parent_rejected() {
    let r = startup_from_file("tests/types/probe_arc237_sB1_recordtype.wat.bad");
    assert!(
        r.is_err(),
        "recordtype with an unknown parent must be rejected at registration; got Ok"
    );
}

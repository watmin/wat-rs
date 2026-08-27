//! FM 2-bis probe — arc 237 Stone S-B.1: `:wat::core::recordtype` + `TypeDef::Record`.
//!
//! Wat source: tests/types/probe_arc237_sB1_recordtype.wat (loaded via startup_beside).
//! Probe 06 (negative) uses tests/types/probe_arc237_sB1_recordtype.wat.bad.

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;
use wat::types::TypeErrorKind;

fn run_bool(fn_name: &str) -> Result<bool, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {:?}", e))? {
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
    wat::assert_startup_error!(r,
        StartupError::Type(e) if matches!(e.kind(), TypeErrorKind::MalformedDecl { head, reason }
            if head == "recordtype"
            && reason == "parent ':my::DoesNotExist' is not a nature-root; inheritance is \
                           unsupported — reuse a shape via surface-splice `[~@:Surface …]`")
    );
}

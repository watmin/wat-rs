//! FM 2-bis probe — arc 237 Stone S-B.2: defrecord emits `recordtype` + drops its predicate.
//!
//! Wat source: tests/types/probe_arc237_sB2_defrecord_recordtype.wat (loaded via startup_beside).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!(&format!("({fn_name})")).map_err(|e| format!("parse: {:?}", e))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn assert_bool(fn_name: &str, want: bool) {
    match run(fn_name) {
        Ok(Value::bool(b)) if b == want => {}
        other => panic!("expected {} for `{}`; got {:?}", want, fn_name, other),
    }
}

// ─── Probe 01: everyday is-X? ∀T — asymmetry dead on the real surface ───────
#[test]
fn probe_01_everyday_is_predicate_forall_t() {
    assert_bool(":user::probe-01", false);
}

// ─── Probe 02: is-X? TRUE-path (B.1-deferred, now provable via the constructor) ─
#[test]
fn probe_02_is_predicate_true_path() {
    assert_bool(":user::probe-02", true);
}

// ─── Probe 03: is-X? cross-class false ──────────────────────────────────────
#[test]
fn probe_03_is_predicate_cross_class_false() {
    assert_bool(":user::probe-03", false);
}

// ─── Probe 04: edge wired by the emitted recordtype ─────────────────────────
#[test]
fn probe_04_edge_wired() {
    assert_bool(":user::probe-04", true);
}

// ─── Probe 05: accessors + constructor still work (regression) ───────────────
#[test]
fn probe_05_accessors_still_work() {
    match run(":user::probe-05") {
        Ok(Value::f64(x)) if (x - 1.0).abs() < 1e-9 => {}
        other => panic!("expected 1.0 from accessor; got {:?}", other),
    }
}

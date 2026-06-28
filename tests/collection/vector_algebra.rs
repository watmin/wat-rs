//! Arc 053 slice 1 — Vector-tier algebra primitives.
//!
//! Coverage: vector-bind, vector-bundle, vector-blend, vector-permute
//! over `Value::Vector` inputs (post-arc-052).
//!
//! Wat source lives in the co-located fixture: vector_algebra.wat
//! (slurped via startup_beside(file!())). Functions return String results
//! so tests use eval_in_frozen rather than stdout capture.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_str(call: &str) -> String {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned()
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
    }
}

#[test]
fn vector_bind_roundtrip() {
    // bind(a, b) == bind(a, b) — deterministic.
    assert_eq!(run_str("(:valg::bind-roundtrip)"), "yes");
}

#[test]
fn vector_bundle_singleton_returns_input() {
    // Bundle of a single vector returns ~the input (sign of the only contributor).
    assert_eq!(run_str("(:valg::bundle-singleton)"), "near-1");
}

#[test]
fn vector_blend_weighted() {
    // blend(a, a, 1.0, 0.0) should equal a.
    assert_eq!(run_str("(:valg::blend-weighted)"), "near-1");
}

#[test]
fn vector_permute_changes_vector() {
    // permute(v, k) for k != 0 should differ from v.
    assert_eq!(run_str("(:valg::permute-changes)"), "differs");
}

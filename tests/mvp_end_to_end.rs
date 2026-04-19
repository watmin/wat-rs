//! MVP end-to-end integration test — source text to vector.
//!
//! The "door works" moment. Given a wat source string containing
//! algebra-core forms, the pipeline (parse → lower → encode) produces
//! a `holon::Vector`. No `define`, no `load!`, no macros, no types —
//! just the substrate-through-source proof.

use holon::{AtomTypeRegistry, ScalarEncoder, VectorManager};
use wat_rs::eval_algebra_source;

const D: usize = 1024;

fn env() -> (VectorManager, ScalarEncoder, AtomTypeRegistry) {
    (
        VectorManager::with_seed(D, 42),
        ScalarEncoder::with_seed(D, 42),
        AtomTypeRegistry::with_builtins(),
    )
}

/// Hello-world: the minimum wat source that proves source → vector works.
const HELLO_WORLD: &str = r#"
(:wat::algebra::Bind
  (:wat::algebra::Atom "role")
  (:wat::algebra::Atom "filler"))
"#;

#[test]
fn hello_world_door_works() {
    let (vm, se, reg) = env();
    let v = eval_algebra_source(HELLO_WORLD, &vm, &se, &reg).unwrap();
    assert_eq!(v.dimensions(), D);
}

#[test]
fn hello_world_is_deterministic() {
    // Same source, two independent environments with the same seed,
    // must produce bit-identical vectors.
    let (vm1, se1, reg1) = env();
    let (vm2, se2, reg2) = env();
    let v1 = eval_algebra_source(HELLO_WORLD, &vm1, &se1, &reg1).unwrap();
    let v2 = eval_algebra_source(HELLO_WORLD, &vm2, &se2, &reg2).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn different_sources_produce_different_vectors() {
    let (vm, se, reg) = env();
    let v1 = eval_algebra_source(
        r#"(:wat::algebra::Atom "role")"#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    let v2 = eval_algebra_source(
        r#"(:wat::algebra::Atom "filler")"#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    assert_ne!(v1, v2);
}

#[test]
fn bind_vs_bundle_of_same_atoms_differ() {
    // (:wat::algebra::Bind a b) and (:wat::algebra::Bundle (:wat::core::vec a b))
    // are different algebra operations; their vectors must differ.
    let (vm, se, reg) = env();
    let v_bind = eval_algebra_source(
        r#"(:wat::algebra::Bind (:wat::algebra::Atom "a") (:wat::algebra::Atom "b"))"#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    let v_bundle = eval_algebra_source(
        r#"(:wat::algebra::Bundle (:wat::core::vec (:wat::algebra::Atom "a") (:wat::algebra::Atom "b")))"#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    assert_ne!(v_bind, v_bundle);
}

#[test]
fn thermometer_endpoints_produce_opposite_vectors() {
    use holon::Similarity;
    let (vm, se, reg) = env();
    let v_low = eval_algebra_source(
        "(:wat::algebra::Thermometer 0.0 0.0 1.0)",
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    let v_high = eval_algebra_source(
        "(:wat::algebra::Thermometer 1.0 0.0 1.0)",
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    // All -1 at min, all +1 at max — cosine ≈ -1.
    let sim = Similarity::cosine(&v_low, &v_high);
    assert!(sim < -0.99, "expected cosine ≈ -1, got {}", sim);
}

#[test]
fn blend_option_b_subtract_literal() {
    // Subtract = Blend(a, b, 1, -1) per 058-002. Literal negative weight
    // flows through parse → lower → encode as expected.
    let (vm, se, reg) = env();
    let v = eval_algebra_source(
        r#"(:wat::algebra::Blend (:wat::algebra::Atom "x") (:wat::algebra::Atom "y") 1 -1)"#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    assert_eq!(v.dimensions(), D);
}

#[test]
fn keyword_atom_differs_from_string_atom() {
    // (Atom :role) and (Atom "role") — the leading ':' in the keyword's
    // stored bytes makes the two vectors differ. Verifies the keyword
    // convention crosses the whole parse → lower → encode pipeline.
    let (vm, se, reg) = env();
    let v_kw = eval_algebra_source("(:wat::algebra::Atom :role)", &vm, &se, &reg).unwrap();
    let v_str =
        eval_algebra_source(r#"(:wat::algebra::Atom "role")"#, &vm, &se, &reg).unwrap();
    assert_ne!(v_kw, v_str);
}

#[test]
fn whitespace_and_comments_ignored() {
    let (vm, se, reg) = env();
    let v1 = eval_algebra_source(HELLO_WORLD, &vm, &se, &reg).unwrap();
    let v2 = eval_algebra_source(
        r#"
        ;; the hello world bind
        (:wat::algebra::Bind
          ;; role
          (:wat::algebra::Atom "role")
          ;; filler
          (:wat::algebra::Atom "filler"))
        "#,
        &vm,
        &se,
        &reg,
    )
    .unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn parse_error_surfaces_as_error() {
    let (vm, se, reg) = env();
    // Unclosed paren.
    let err = eval_algebra_source("(:wat::algebra::Atom \"x\"", &vm, &se, &reg).unwrap_err();
    match err {
        wat_rs::Error::Parse(_) => {} // expected
        other => panic!("expected ParseError, got {:?}", other),
    }
}

#[test]
fn lower_error_surfaces_as_error() {
    let (vm, se, reg) = env();
    // Unsupported algebra-core form.
    let err =
        eval_algebra_source("(:wat::algebra::MadeUp 1)", &vm, &se, &reg).unwrap_err();
    match err {
        wat_rs::Error::Lower(_) => {} // expected
        other => panic!("expected LowerError, got {:?}", other),
    }
}

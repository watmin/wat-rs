//! MVP end-to-end integration test — source text to vector.
//!
//! The "door works" moment. Given a wat source string containing
//! algebra-core forms, the pipeline (parse → lower → encode) produces
//! a `holon::Vector`. No `define`, no `load!`, no macros, no types —
//! just the substrate-through-source proof.

use holon::{ScalarEncoder, VectorManager};
use wat::eval_algebra_source;

const D: usize = 1024;

fn env() -> (VectorManager, ScalarEncoder) {
    (
        VectorManager::with_seed(D, 42),
        ScalarEncoder::with_seed(D, 42),
    )
}

/// Hello-world: the minimum wat source that proves source → vector works.
const HELLO_WORLD: &str = r#"
(:wat::holon::Bind
  (:wat::holon::Atom "role")
  (:wat::holon::Atom "filler"))
"#;

#[test]
fn hello_world_door_works() {
    let (vm, se) = env();
    let v = eval_algebra_source(HELLO_WORLD, &vm, &se).unwrap();
    assert_eq!(v.dimensions(), D);
}

#[test]
fn hello_world_is_deterministic() {
    // Same source, two independent environments with the same seed,
    // must produce bit-identical vectors.
    let (vm1, se1) = env();
    let (vm2, se2) = env();
    let v1 = eval_algebra_source(HELLO_WORLD, &vm1, &se1).unwrap();
    let v2 = eval_algebra_source(HELLO_WORLD, &vm2, &se2).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn different_sources_produce_different_vectors() {
    let (vm, se) = env();
    let v1 = eval_algebra_source(
        r#"(:wat::holon::Atom "role")"#,
        &vm,
        &se,
    )
    .unwrap();
    let v2 = eval_algebra_source(
        r#"(:wat::holon::Atom "filler")"#,
        &vm,
        &se,
    )
    .unwrap();
    assert_ne!(v1, v2);
}

#[test]
fn bind_vs_bundle_of_same_atoms_differ() {
    // (:wat::holon::Bind a b) and (:wat::holon::Bundle (:wat::core::Vectora b))
    // are different algebra operations; their vectors must differ.
    let (vm, se) = env();
    let v_bind = eval_algebra_source(
        r#"(:wat::holon::Bind (:wat::holon::Atom "a") (:wat::holon::Atom "b"))"#,
        &vm,
        &se,
    )
    .unwrap();
    let v_bundle = eval_algebra_source(
        r#"(:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST (:wat::holon::Atom "a") (:wat::holon::Atom "b")))"#,
        &vm,
        &se,
    )
    .unwrap();
    assert_ne!(v_bind, v_bundle);
}

#[test]
fn thermometer_endpoints_produce_opposite_vectors() {
    use holon::Similarity;
    let (vm, se) = env();
    let v_low = eval_algebra_source(
        "(:wat::holon::Thermometer 0.0 0.0 1.0)",
        &vm,
        &se,
    )
    .unwrap();
    let v_high = eval_algebra_source(
        "(:wat::holon::Thermometer 1.0 0.0 1.0)",
        &vm,
        &se,
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
    let (vm, se) = env();
    let v = eval_algebra_source(
        r#"(:wat::holon::Blend (:wat::holon::Atom "x") (:wat::holon::Atom "y") 1 -1)"#,
        &vm,
        &se,
    )
    .unwrap();
    assert_eq!(v.dimensions(), D);
}

#[test]
fn keyword_atom_differs_from_string_atom() {
    // (Atom :role) and (Atom "role") — the leading ':' in the keyword's
    // stored bytes makes the two vectors differ. Verifies the keyword
    // convention crosses the whole parse → lower → encode pipeline.
    let (vm, se) = env();
    let v_kw = eval_algebra_source("(:wat::holon::Atom :role)", &vm, &se).unwrap();
    let v_str =
        eval_algebra_source(r#"(:wat::holon::Atom "role")"#, &vm, &se).unwrap();
    assert_ne!(v_kw, v_str);
}

#[test]
fn whitespace_and_comments_ignored() {
    let (vm, se) = env();
    let v1 = eval_algebra_source(HELLO_WORLD, &vm, &se).unwrap();
    let v2 = eval_algebra_source(
        r#"
        ;; the hello world bind
        (:wat::holon::Bind
          ;; role
          (:wat::holon::Atom "role")
          ;; filler
          (:wat::holon::Atom "filler"))
        "#,
        &vm,
        &se,
    )
    .unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn parse_error_surfaces_as_error() {
    let (vm, se) = env();
    // Unclosed paren.
    let err = eval_algebra_source("(:wat::holon::Atom \"x\"", &vm, &se).unwrap_err();
    match err {
        wat::Error::Parse(_) => {} // expected
        other => panic!("expected ParseError, got {:?}", other),
    }
}

#[test]
fn lower_error_surfaces_as_error() {
    let (vm, se) = env();
    // Unsupported algebra-core form.
    let err =
        eval_algebra_source("(:wat::holon::MadeUp 1)", &vm, &se).unwrap_err();
    match err {
        wat::Error::Lower(_) => {} // expected
        other => panic!("expected LowerError, got {:?}", other),
    }
}

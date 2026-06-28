//! Arc 052 — Vector as a first-class wat-tier value.
//!
//! Coverage:
//! - Construct via `(:wat::holon::encode <ast>)`
//! - Equality (bit-exact, dim-aware)
//! - Vector as struct field (round-trip through field access)
//! - Polymorphic cosine: AST-AST, Vector-Vector, mixed
//! - Polymorphic dot: same surface as cosine
//! - Polymorphic simhash: AST input vs Vector input agree
//! - Type system: cosine accepts EDN-representable types (arc 294.a)
//! - Determinism: encode is reproducible
//!
//! Wat source lives in the co-located fixture: vector_first_class.wat
//! (slurped via startup_beside(file!())). Functions return String/f64 results
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

// ─── Construct + equality ────────────────────────────────────────────

#[test]
fn vector_construct_via_encode() {
    assert_eq!(run_str("(:vfc::construct-via-encode)"), "equal");
}

#[test]
fn vector_distinct_atoms_distinct_vectors() {
    assert_eq!(run_str("(:vfc::distinct-atoms)"), "diff");
}

// ─── Vector as struct field ─────────────────────────────────────────

#[test]
fn vector_as_struct_field_roundtrip() {
    assert_eq!(run_str("(:vfc::struct-field-roundtrip)"), "yes");
}

// ─── Polymorphic cosine — all four argument shapes ──────────────────

#[test]
fn polymorphic_cosine_ast_ast() {
    // Existing behavior preserved.
    assert_eq!(run_str("(:vfc::cosine-ast-ast)"), "near-1");
}

#[test]
fn polymorphic_cosine_vector_vector() {
    assert_eq!(run_str("(:vfc::cosine-vec-vec)"), "near-1");
}

#[test]
fn polymorphic_cosine_ast_vector_mixed() {
    assert_eq!(run_str("(:vfc::cosine-ast-vec)"), "near-1");
}

#[test]
fn polymorphic_cosine_vector_ast_mixed() {
    assert_eq!(run_str("(:vfc::cosine-vec-ast)"), "near-1");
}

// ─── Polymorphic dot — Vector pair ──────────────────────────────────

#[test]
fn polymorphic_dot_vector_vector() {
    assert_eq!(run_str("(:vfc::dot-vec-vec)"), "positive");
}

// ─── Polymorphic simhash — AST and Vector inputs agree ──────────────

#[test]
fn polymorphic_simhash_ast_and_vector_agree() {
    assert_eq!(run_str("(:vfc::simhash-agree)"), "same");
}

// ─── Type system: cosine accepts EDN-representable types (arc 294.a) ───────────

// Arc 294.a — UPDATED: cosine now accepts any EdnRepresentable value, lifting via
// to_holon_inner. String IS EDN-representable (portable); the old type rejection
// was the inversion 294.a annihilates. The fixture defn :vfc::cosine-string exists
// to force type-checking at startup_beside time; startup succeeding proves acceptance.
#[test]
fn polymorphic_cosine_accepts_string() {
    // startup_beside type-checks the fixture including :vfc::cosine-string.
    // If cosine rejects String args, startup fails here.
    let world = startup_beside(file!());
    assert!(
        world.is_ok(),
        "cosine on string args must now succeed (String is EDN-representable); got: {:?}",
        world.err()
    );
}

// ─── Determinism: encode is reproducible ────────────────────────────

#[test]
fn vector_encode_deterministic_across_calls() {
    // Two encodes of an identical compound AST → equal Vectors.
    assert_eq!(run_str("(:vfc::encode-deterministic)"), "deterministic");
}

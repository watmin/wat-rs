//! FM 2-bis probe — arc 237 Stone 237.5: `:wat::core::conforms?` general type-conformance primitive.
//!
//! `conforms?` is THE type-conformance mechanism — one recursive function over the
//! TypeExpr grammar (Path / Parametric / Tuple / + alias-resolve + union-membership).
//! `is-<Name>?` (Stone 237.6) composes over it. Per memory `feedback_conforms_is_foundation`
//! + arc 237 DESIGN § Reshaped downstream stones.
//!
//! Signature: `(:wat::core::conforms? <value> :TypeExpr) -> :wat::core::bool`
//!   - nominal Path (record/primitive) → identity check (value's tag == name)
//!   - Path → Union           → membership (value conforms to ANY member)
//!   - Path → Alias           → resolve to target, recurse
//!   - Parametric (Vector<T>) → classifier match + recurse element-wise
//!   - well-formed type, no match → false ;  unknown/Fn/Var type → ERROR (not false)
//!
//! Probe contracts (12):
//!   1.  record conforms its own type → true
//!   2.  record does NOT conform a different record → false
//!   3.  i64 value conforms :i64 → true ; conforms :f64 → false
//!   4.  u8 value conforms :u8 → true ; conforms :i64 → false   (NON-ERASURE: u8 ≠ i64 at runtime)
//!   5.  union member conforms the union → true
//!   6.  non-member does NOT conform the union → false
//!   7.  primitive-member union: i64 conforms :Numeric → true
//!   8.  structural Vector<u8>: all-u8 vector → true
//!   9.  structural Vector<u8>: i64-vector → false  (element check recurses)
//!   10. alias resolves: u8-vector conforms :Bytes (= Vector<u8>) → true
//!   11. nested Vector<Shape> (Shape a union): vector of members → true
//!   12. error contract: conforms? to an UNKNOWN type name → Err (not false)
//!
//! Initial state: file fails — `:wat::core::conforms?` does not exist.
//! Post-stone 237.5: 12/12 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites this
//! file verbatim as "the working contract sonnet must satisfy."

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

/// Shared type declarations.
const PRELUDE: &str = r#"
(:wat::core::defrecord :my::Circle [radius <- :wat::core::f64])
(:wat::core::defrecord :my::Square [side <- :wat::core::f64])
(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])
(:wat::core::typeunion :my::Numeric [:wat::core::i64 :wat::core::f64])
(:wat::core::typealias :my::Bytes :wat::core::Vector<wat::core::u8>)
"#;

/// Build `PRELUDE + (:user::compute -> :bool <expr>) + main`, evaluate
/// `(:user::compute)`, return its Value (expected `Value::bool`) or an Err string.
fn run_bool(compute_expr: &str) -> Result<Value, String> {
    let full = format!(
        "{prelude}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::bool {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        prelude = PRELUDE,
        expr = compute_expr
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn assert_true(expr: &str) {
    match run_bool(expr) {
        Ok(Value::bool(true)) => {}
        other => panic!("expected conforms? true for `{}`; got {:?}", expr, other),
    }
}

fn assert_false(expr: &str) {
    match run_bool(expr) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected conforms? false for `{}`; got {:?}", expr, other),
    }
}

// A u8 vector, an i64 vector — reused across structural contracts.
const U8_VEC: &str =
    "(:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3))";
const I64_VEC: &str = "(:wat::core::Vector :wat::core::i64 1 2 3)";

// ─── Probe 1–2: nominal record identity ───────────────────────────────────────

#[test]
fn probe_01_record_conforms_self() {
    assert_true("(:wat::core::conforms? (:my::Circle 1.0) :my::Circle)");
}

#[test]
fn probe_02_record_not_conforms_other() {
    assert_false("(:wat::core::conforms? (:my::Circle 1.0) :my::Square)");
}

// ─── Probe 3: nominal primitive identity ───────────────────────────────────────

#[test]
fn probe_03_primitive_i64_identity() {
    assert_true("(:wat::core::conforms? 1 :wat::core::i64)");
    assert_false("(:wat::core::conforms? 1 :wat::core::f64)");
}

// ─── Probe 4: u8 ≠ i64 at runtime (non-erasure, end-to-end) ───────────────────

#[test]
fn probe_04_u8_distinct_from_i64() {
    assert_true("(:wat::core::conforms? (:wat::core::u8 1) :wat::core::u8)");
    assert_false("(:wat::core::conforms? (:wat::core::u8 1) :wat::core::i64)");
}

// ─── Probe 5–7: union membership ───────────────────────────────────────────────

#[test]
fn probe_05_union_member_true() {
    assert_true("(:wat::core::conforms? (:my::Circle 1.0) :my::Shape)");
}

#[test]
fn probe_06_union_non_member_false() {
    assert_false("(:wat::core::conforms? 1 :my::Shape)");
}

#[test]
fn probe_07_primitive_member_union() {
    assert_true("(:wat::core::conforms? 1 :my::Numeric)");
    assert_false("(:wat::core::conforms? \"x\" :my::Numeric)");
}

// ─── Probe 8–9: structural Vector<u8> ──────────────────────────────────────────

#[test]
fn probe_08_structural_vector_u8_true() {
    assert_true(&format!(
        "(:wat::core::conforms? {} :wat::core::Vector<wat::core::u8>)",
        U8_VEC
    ));
}

#[test]
fn probe_09_structural_vector_u8_false_on_i64_elements() {
    assert_false(&format!(
        "(:wat::core::conforms? {} :wat::core::Vector<wat::core::u8>)",
        I64_VEC
    ));
}

// ─── Probe 10: alias resolves to its target ────────────────────────────────────

#[test]
fn probe_10_alias_resolves() {
    assert_true(&format!("(:wat::core::conforms? {} :my::Bytes)", U8_VEC));
    assert_false(&format!("(:wat::core::conforms? {} :my::Bytes)", I64_VEC));
}

// ─── Probe 11: nested Vector<Shape> (union-in-element) ─────────────────────────

#[test]
fn probe_11_nested_vector_of_union() {
    let shape_vec =
        "(:wat::core::Vector :my::Shape (:my::Circle 1.0) (:my::Square 2.0))";
    assert_true(&format!(
        "(:wat::core::conforms? {} :wat::core::Vector<my::Shape>)",
        shape_vec
    ));
    // An i64-vector does not conform to Vector<Shape>.
    assert_false(&format!(
        "(:wat::core::conforms? {} :wat::core::Vector<my::Shape>)",
        I64_VEC
    ));
}

// ─── Probe 12: error contract — unknown type name is an ERROR, not false ───────

#[test]
fn probe_12_unknown_type_name_errors() {
    let r = run_bool("(:wat::core::conforms? 1 :my::DoesNotExist)");
    assert!(
        r.is_err(),
        "conforms? against an unknown type name must error (bad input), not return false; got {:?}",
        r
    );
}

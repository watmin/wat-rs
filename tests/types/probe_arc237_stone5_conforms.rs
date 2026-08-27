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

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeErrorKind, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

fn run_bool(fn_name: &str) -> Result<Value, String> {
    call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {:?}", e))
}

fn assert_true(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(true)) => {}
        other => panic!("expected conforms? true for `{}`; got {:?}", fn_name, other),
    }
}

fn assert_false(fn_name: &str) {
    match run_bool(fn_name) {
        Ok(Value::bool(false)) => {}
        other => panic!("expected conforms? false for `{}`; got {:?}", fn_name, other),
    }
}

// ─── Probe 1–2: nominal record identity ───────────────────────────────────────

#[test]
fn probe_01_record_conforms_self() {
    assert_true(":user::probe01");
}

#[test]
fn probe_02_record_not_conforms_other() {
    assert_false(":user::probe02");
}

// ─── Probe 3: nominal primitive identity ───────────────────────────────────────

#[test]
fn probe_03_primitive_i64_identity() {
    assert_true(":user::probe03a");
    assert_false(":user::probe03b");
}

// ─── Probe 4: u8 ≠ i64 at runtime (non-erasure, end-to-end) ───────────────────

#[test]
fn probe_04_u8_distinct_from_i64() {
    assert_true(":user::probe04a");
    assert_false(":user::probe04b");
}

// ─── Probe 5–7: union membership ───────────────────────────────────────────────

#[test]
fn probe_05_union_member_true() {
    assert_true(":user::probe05");
}

#[test]
fn probe_06_union_non_member_false() {
    assert_false(":user::probe06");
}

#[test]
fn probe_07_primitive_member_union() {
    assert_true(":user::probe07a");
    assert_false(":user::probe07b");
}

// ─── Probe 8–9: structural Vector<u8> ──────────────────────────────────────────

#[test]
fn probe_08_structural_vector_u8_true() {
    assert_true(":user::probe08");
}

#[test]
fn probe_09_structural_vector_u8_false_on_i64_elements() {
    assert_false(":user::probe09");
}

// ─── Probe 10: alias resolves to its target ────────────────────────────────────

#[test]
fn probe_10_alias_resolves() {
    assert_true(":user::probe10a");
    assert_false(":user::probe10b");
}

// ─── Probe 11: nested Vector<Shape> (union-in-element) ─────────────────────────

#[test]
fn probe_11_nested_vector_of_union() {
    assert_true(":user::probe11a");
    // An i64-vector does not conform to Vector<Shape>.
    assert_false(":user::probe11b");
}

// ─── Probe 12: error contract — unknown type name is an ERROR, not false ───────

#[test]
fn probe_12_unknown_type_name_errors() {
    // Bypasses `run_bool` (which formats the error to a bare String) — the discriminant
    // needs the structured `RuntimeError` (arc 296 Stone L: a bare `is_err()` is satisfied
    // by ANY error, including a retirement or a renamed fixture, not just the declared one).
    let r = call_beside_value(file!(), ":user::probe12");
    assert!(
        matches!(&r, Err(e) if matches!(e.kind(), RuntimeErrorKind::MalformedForm { head, reason }
            if head == ":wat::core::conforms?"
            && reason == "unknown type name ':my::DoesNotExist' is not registered in the TypeEnv \
                           and is not a built-in primitive; cannot determine conformance (this is \
                           bad input, not a negative result — check the spelling and ensure the \
                           type is declared before use)")),
        "conforms? against an unknown type name must error (bad input), not return false; got {:?}",
        r
    );
}

//! Integration tests for `:wat::core::do` — Clojure-faithful sequential
//! evaluation form. Arc 136 slice 1a.
//!
//! Shape: `(:wat::core::do f1 f2 ... fN)`.
//!
//! Semantics:
//!   - Variadic; one or more forms.
//!   - Empty `(do)` → MalformedForm parse error.
//!   - Each non-final form is evaluated for side effect; its result is
//!     DISCARDED. Non-finals' types are unconstrained.
//!   - The FINAL form is evaluated; its value is returned.
//!   - The do form's inferred type IS the final form's inferred type.
//!     Recipient unification at the consuming site (binding slot,
//!     function declared return, argument position) is the static check.
//!
//! No `-> :T` slot — per the FOURTH amendment to the arc 136 DESIGN
//! (and the arc 145 back-out realization), the substrate's existing
//! inference + recipient unification provides the static check.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn unwrap_i64(v: Value) -> i64 {
    match v {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn run_err_file(rel_path: &str) -> String {
    match startup_from_file(rel_path) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── 1. Empty: (:wat::core::do) → MalformedForm parse error ─────────────

#[test]
fn do_empty_form_is_malformed() {
    let err = run_err_file("tests/wat_lang/wat_arc136_do_form_empty.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc136_do_form__do_empty_form_is_malformed.edn",
        "expected MalformedForm naming the do form"
    );
}

// ─── 2. Single form: (do x) ≡ x ─────────────────────────────────────────

#[test]
fn do_single_form_returns_its_value() {
    assert_eq!(unwrap_i64(run_expr(":t::test2-single")), 42);
}

// ─── 3. Multi form: side effects in order; final value returned ─────────

#[test]
fn do_multi_form_evaluates_left_to_right_returns_final() {
    assert_eq!(unwrap_i64(run_expr(":t::test3-multi")), 99);
}

// ─── 4. Type flow at recipient (clean unification) ──────────────────────

#[test]
fn do_recipient_unifies_with_final_form_type() {
    assert_eq!(unwrap_i64(run_expr(":t::test4-recipient")), 42);
}

// ─── 5. Recipient mismatch fires TypeMismatch ───────────────────────────

#[test]
fn do_recipient_mismatch_fires_type_mismatch() {
    let err = run_err_file("tests/wat_lang/wat_arc136_do_form_mismatch.wat.bad");
    wat::assert_edn_matches_file!(
        err,
        "wat_arc136_do_form__do_recipient_mismatch_fires_type_mismatch.edn",
        "expected TypeMismatch at probe's body"
    );
}

// ─── 6. Non-final type unconstrained ────────────────────────────────────

#[test]
fn do_non_final_type_is_unconstrained() {
    assert_eq!(unwrap_i64(run_expr(":t::test6-non-final")), 42);
}

// ─── 7. Reflection round-trip via signature-of-defn ─────────────────────

#[test]
fn do_reflection_round_trip_emits_variadic_sketch() {
    let rendered = unwrap_string(run_expr(":t::test7-signature"));
    wat::assert_edn_matches_file!(rendered, "wat_arc136_do_form__do_signature.edn", "expected do keyword as signature head with variadic <form>+ slot");
}

// ─── 8. Tail-call sanity: do in tail position preserves TCO ─────────────

#[test]
fn do_in_tail_position_preserves_tail_call() {
    assert_eq!(unwrap_i64(run_expr(":t::test8-tail")), 0);
}

// ─── 9. Nested do forms compose ─────────────────────────────────────────

#[test]
fn do_nested_compose_cleanly() {
    assert_eq!(unwrap_i64(run_expr(":t::test9-nested")), 2);
}

// ─── 10. Mixed with let: types compose ─────────────────────────────────

#[test]
fn do_inside_let_body_composes_types_cleanly() {
    assert_eq!(unwrap_i64(run_expr(":t::test10-let-body")), 7);
}

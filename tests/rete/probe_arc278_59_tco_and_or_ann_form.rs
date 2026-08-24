//! Arc 278 #59 — `and`, `or`, `ann-form` mirror the `if`/`match`/`let`/`do` pattern `eval_tail`
//! already dispatches to their `*_tail` TCO variants. Contract:
//! `docs/arc/2026/06/278-rules-engine/BRIEF-tco-and-or-ann-form.md`.
//!
//! THE GATE THAT DECIDES WHETHER THIS SHIPPED (per the brief): each TCO test below must go RED
//! without its arm. That toggle (land the arm, watch the test die with the arm removed, restore
//! it) is a manual one-time verification reported in prose in the rider's report — same discipline
//! `probe_arc278_55_slice_one_vocabulary.rs` uses for its own `if` TCO gate (there is no safe way
//! to "expect" a SIGSEGV/SIGABRT from inside the process it would kill).
//!
//! Run: cargo test --release --test rete

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeErrorKind, Value};

fn is_true(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(true)) }
fn eval_i64(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64, got {other:?}"),
    }
}
/// Extract the `Ok` i64 payload of a `:wat::core::Result<wat::core::i64,wat::core::EvalError>`
/// entry, panicking with the full value on anything else (including `Err`) so a failure names
/// what actually came back rather than just "not Ok".
fn eval_result_ok_i64(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).expect("eval") {
        Value::Result(r) => match &*r {
            Ok(Value::i64(n)) => *n,
            other => panic!("entry {entry:?}: expected Ok(i64), got {other:?}"),
        },
        other => panic!("entry {entry:?}: expected Value::Result, got {other:?}"),
    }
}

// ─── THE TCO GATE — depth 150000 ────────────────────────────────────────────────

/// `and` in tail position (last operand a self-recursive call) survives depth 150000 —
/// reachable only if every recursive call reused the same native stack frame via
/// `eval_and_tail`. RED-WITHOUT-ARM OBSERVATION (manual, reported in prose): with
/// `":wat::core::and" => eval_and_tail(...)` removed from `eval_tail`'s match, this test binary
/// aborts (SIGABRT, "fatal runtime error: stack overflow") well before returning.
#[test]
fn and_tail_position_preserves_tco_at_depth() {
    assert!(is_true(":user::and-tail-tco-survives-depth"), "the base case's value, reachable only if 150000 tail calls all reused the same native stack frame");
}

/// Sibling of the above for `or`. RED-WITHOUT-ARM OBSERVATION (manual): identical SIGABRT with
/// the `":wat::core::or" => eval_or_tail(...)` arm removed.
#[test]
fn or_tail_position_preserves_tco_at_depth() {
    assert!(!is_true(":user::or-tail-tco-survives-depth"), "the base case's value (false), reachable only if 150000 tail calls all reused the same native stack frame");
}

/// Sibling of the above for `ann-form` (the wrapped expression is the self-recursive call).
/// RED-WITHOUT-ARM OBSERVATION (manual): identical SIGABRT with the
/// `":wat::core::ann-form" => eval_ann_form_tail(...)` arm removed.
#[test]
fn ann_form_tail_position_preserves_tco_at_depth() {
    assert_eq!(eval_i64(":user::ann-form-tail-tco-survives-depth"), 0, "the base case's value, reachable only if 150000 tail calls all reused the same native stack frame");
}

// ─── STOP-3 control: short-circuit survives in tail position ───────────────────────

/// `and` in tail position still short-circuits: a false first operand must never evaluate the
/// tail-called, raising second operand.
#[test]
fn and_tail_still_short_circuits() {
    assert!(!is_true(":user::and-tail-short-circuits"), "a false first operand must return false WITHOUT evaluating the raising tail-called second operand");
}

/// Non-vacuity control for the test above: the identical raising operand, actually reached
/// (first operand true), DOES raise. Matched on the typed KIND, not a rendered substring
/// (`no_loose_string_assert`).
#[test]
fn and_tail_control_the_same_operand_reached_does_raise() {
    let err = call_beside_value(file!(), ":user::and-tail-control-raises")
        .expect_err("a true first operand MUST reach the second and raise — otherwise the short-circuit test above is vacuous");
    assert!(matches!(err.kind(), RuntimeErrorKind::DivisionByZero), "the control must fail on the divide itself; got kind: {:?}", err.kind());
}

/// `or` in tail position still short-circuits: a true first operand must never evaluate the
/// tail-called, raising second operand.
#[test]
fn or_tail_still_short_circuits() {
    assert!(is_true(":user::or-tail-short-circuits"), "a true first operand must return true WITHOUT evaluating the raising tail-called second operand");
}

/// Non-vacuity control for the test above.
#[test]
fn or_tail_control_the_same_operand_reached_does_raise() {
    let err = call_beside_value(file!(), ":user::or-tail-control-raises")
        .expect_err("a false first operand MUST reach the second and raise — otherwise the short-circuit test above is vacuous");
    assert!(matches!(err.kind(), RuntimeErrorKind::DivisionByZero), "the control must fail on the divide itself; got kind: {:?}", err.kind());
}

// ─── STOP-1 control: TCO must not change any answer ────────────────────────────────

/// A shallow, non-recursive `and` answers exactly as `eval_and` would.
#[test]
fn and_tail_shallow_answer_is_unchanged() {
    assert!(!is_true(":user::and-tail-shallow-answer"), "true and true and false = false");
}

/// A shallow, non-recursive `or` answers exactly as `eval_or` would.
#[test]
fn or_tail_shallow_answer_is_unchanged() {
    assert!(is_true(":user::or-tail-shallow-answer"), "false or false or true = true");
}

/// A shallow `ann-form` answers exactly as `eval_ann_form` would (the type slot erased, the
/// wrapped expression's value returned untouched).
#[test]
fn ann_form_tail_shallow_answer_is_unchanged() {
    assert_eq!(eval_i64(":user::ann-form-tail-shallow-answer"), 5);
}

// ─── the RULED weakening, PINNED ────────────────────────────────────────────────

/// Arc 278 #59 obligation #2: pin the traded-away runtime bool check on `and`/`or`'s tail-called
/// last operand, on the one path where it's observable — a `:wat::core::fn` literal built inside
/// `quote` (never type-checked) and invoked via `:wat::eval-ast!` + `:wat::core::apply`. The last
/// operand (`5`, not a bool) is tail-called away instead of raising `TypeMismatch`, so calling the
/// fn returns `Ok(5)` rather than `Err(EvalError{TypeMismatch, ..})`.
///
/// If `eval_and_tail`'s arm were removed from `eval_tail`'s match, this comes back `Err` instead
/// (verified by hand alongside the TCO removal cycle — see the module doc).
#[test]
fn and_tail_skips_the_last_operand_bool_check_on_the_unchecked_eval_ast_path() {
    assert_eq!(
        eval_result_ok_i64(":t::and-tail-skips-last-check"),
        5,
        "the RULED weakening: eval_and_tail tail-calls the last operand (5) away instead of raising TypeMismatch on it"
    );
}

/// Sibling of the above for `or` (last operand `7`, not a bool).
#[test]
fn or_tail_skips_the_last_operand_bool_check_on_the_unchecked_eval_ast_path() {
    assert_eq!(
        eval_result_ok_i64(":t::or-tail-skips-last-check"),
        7,
        "the RULED weakening: eval_or_tail tail-calls the last operand (7) away instead of raising TypeMismatch on it"
    );
}

//! FM 2-bis probe — arc 237 Stone 237.3: `:guard` + `:ensure` clause-keywords.
//!
//! Verifies the load-bearing contract: defclause clauses gain optional `:guard`
//! (boolean expression in clause-arg scope; false → skip clause) + `:ensure`
//! (1-arity :fn taking declared return type, returns :bool; false → raises
//! postcondition error).
//!
//! Stone 237.2 (defclause foundation) shipped at bdd9eb6c with 12/12 PASS.
//! Stone 237.3 LAYERS guards + ensures on top — purely additive; clauses
//! without :guard / :ensure continue to dispatch via arity+type only.
//!
//! Doctrine (per docs/arc/2026/05/237-polymorphism-consolidation/ + scratch
//! 017 ADDENDUM):
//!   - Keyword order FIXED: args → :guard? → :ensure? → body
//!   - ONE :guard per clause (compose multiple conditions with :and; verbose-is-honest)
//!   - ONE :ensure per clause (explicit :fn; new binding for return)
//!   - :guard evaluates in clause-arg scope; false → SKIP clause; runtime error → propagate
//!   - :ensure evaluates AFTER body; false → raise (temporary error; Stone 237.4 refines)
//!
//! Probe contracts (14):
//!   1.  Single clause with :guard true; body fires
//!   2.  Single clause with :guard false; runtime error (no matching clause)
//!   3.  Two clauses, first :guard false; second :guard true; second fires
//!   4.  Factorial demo (3 clauses, all with :guard) — n=0 base case, n>0 recursive
//!   5.  :guard expr non-boolean (returns :i64): type-check error
//!   6.  :ensure :fn returning true: result returned
//!   7.  :ensure :fn returning false: postcondition error raised
//!   8.  :ensure :fn with wrong arity (2 args): type-check error
//!   9.  :ensure :fn arg type mismatch with declared return: type-check error
//!   10. :ensure :fn return type not :bool: type-check error
//!   11. Clause with BOTH :guard and :ensure (full shape)
//!   12. Multiple :guard in same clause: parse-time rejection
//!   13. :ensure BEFORE :guard (order violation): parse-time rejection
//!   14. Complex demo from scratch 017 ADDENDUM (2 same-arity guards + 3-arity with ensure)
//!
//! Initial state: file does not compile cleanly OR tests fail at runtime
//! (defclause currently parses :guard / :ensure as part-of-body or unknown
//! forms; no enforcement of order; no postcondition machinery).
//!
//! Post-stone 237.3: 14/14 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites
//! verbatim as "the working contract sonnet must satisfy."

//! Wat source: tests/function/probe_arc237_stone3_guard_ensure.wat
//! Negative fixtures: probe_arc237_stone3_p05.wat.bad, probe_arc237_stone3_p08.wat.bad,
//!   probe_arc237_stone3_p09.wat.bad, probe_arc237_stone3_p10.wat.bad,
//!   probe_arc237_stone3_p12.wat.bad, probe_arc237_stone3_p13.wat.bad.
//! Runtime-error fns in main fixture: :user::probe-02-err, :user::probe-07-err.

use wat::check::error::{CheckErrorKind, EnsureFnInvalidReason};
use wat::freeze::{startup_beside, startup_from_file, StartupError};
use wat::runtime::{apply_function, ClauseFailureReason, RuntimeErrorKind, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for stone3 guard-ensure fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

/// Fetch + apply a zero-arg fn from the shared sibling fixture, returning the raw
/// `Result` (rather than `expect`ing) so error-path probes can assert `is_err()`.
fn try_run(fn_name: &str) -> Result<Value, wat::runtime::RuntimeError> {
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no {fn_name} in fixture")).clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_01_guard_true_body_fires() {
    assert_eq!(run(":user::probe-01"), Value::i64(42), ":guard true should allow body to fire → 42");
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_guard_false_no_match_runtime_error() {
    // :guard false on the only clause → NoMatchingClause at RUNTIME (startup succeeds).
    let result = try_run(":user::probe-02-err");
    assert!(
        matches!(
            &result,
            Err(e) if matches!(
                e.kind(),
                RuntimeErrorKind::NoMatchingClause { name, called_arity, attempted_clauses, .. }
                    if name == ":p02::pick"
                    && *called_arity == 1
                    && attempted_clauses.len() == 1
                    && attempted_clauses[0].clause_index == 0
                    && matches!(&attempted_clauses[0].failure_reason, ClauseFailureReason::GuardFalse)
            )
        ),
        ":guard false on the only clause should raise RuntimeErrorKind::NoMatchingClause{{name: \":p02::pick\", clause 0: GuardFalse}}; got {:?}",
        result
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_guard_false_falls_through_to_next_clause() {
    // first guard (x > 100) is false for 42; second guard (x > 0) is true; second body fires.
    assert_eq!(
        run(":user::probe-03"),
        Value::i64(42),
        "first guard (x > 100) false for 42; second guard (x > 0) true; second body fires",
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_factorial_demo_via_guards() {
    // Per scratch 017 ADDENDUM Demo 1 — Factorial (Erlang spirit via Path C).
    // 5! = 120.
    assert_eq!(run(":user::probe-04"), Value::i64(120), "factorial(5) = 120");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_guard_non_boolean_errors_at_check() {
    // :guard must produce :bool. An :i64 expression should fail type-check.
    let result = startup_from_file("tests/function/probe_arc237_stone3_p05.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::GuardExprNotBoolean { defclause_name, clause_index, got_type }
            if defclause_name == ":my::bad"
            && *clause_index == 0
            && got_type == ":wat::core::i64"
    );
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_ensure_true_returns_result() {
    assert_eq!(run(":user::probe-06"), Value::i64(42), ":ensure true should return result → 42");
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_ensure_false_raises_postcondition() {
    // :ensure false (result -5 not > 0) → postcondition error at RUNTIME (startup succeeds).
    let result = try_run(":user::probe-07-err");
    assert!(
        matches!(
            &result,
            Err(e) if matches!(
                e.kind(),
                RuntimeErrorKind::PostconditionFailed { defclause_name, clause_index, returned_value, .. }
                    if defclause_name == ":p07::positive"
                    && *clause_index == 0
                    && returned_value.rendered == "-5"
            )
        ),
        ":ensure false (result -5 not > 0) should raise RuntimeErrorKind::PostconditionFailed{{defclause: \":p07::positive\", clause 0, returned -5}}; got {:?}",
        result
    );
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_ensure_fn_wrong_arity_errors_at_check() {
    // :ensure :fn must be 1-arity. 2-arity should reject at type-check.
    let result = startup_from_file("tests/function/probe_arc237_stone3_p08.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason }
            if defclause_name == ":my::bad"
            && *clause_index == 0
            && matches!(reason, EnsureFnInvalidReason::ArityNotOne { got } if *got == 2)
    );
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_ensure_fn_arg_type_mismatch_errors_at_check() {
    // :ensure :fn's arg type must match the clause's declared return type.
    let result = startup_from_file("tests/function/probe_arc237_stone3_p09.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason }
            if defclause_name == ":my::bad"
            && *clause_index == 0
            && matches!(
                reason,
                EnsureFnInvalidReason::ArgTypeMismatch { arg_type, clause_return_type }
                    if arg_type == ":wat::core::String" && clause_return_type == ":wat::core::i64"
            )
    );
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_ensure_fn_return_not_bool_errors_at_check() {
    let result = startup_from_file("tests/function/probe_arc237_stone3_p10.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason }
            if defclause_name == ":my::bad"
            && *clause_index == 0
            && matches!(reason, EnsureFnInvalidReason::ReturnTypeNotBool { got } if got == ":wat::core::i64")
    );
}

// ─── Probe 11 ───────────────────────────────────────────────────────────────
#[test]
fn probe_11_full_shape_guard_and_ensure() {
    // Both :guard AND :ensure in one clause. guard true + ensure true → i64(42).
    assert_eq!(
        run(":user::probe-11"),
        Value::i64(42),
        "guard true + ensure true should return result → 42",
    );
}

// ─── Probe 12 ───────────────────────────────────────────────────────────────
#[test]
fn probe_12_multiple_guards_rejected() {
    // ONE :guard per clause; multiple should reject.
    let result = startup_from_file("tests/function/probe_arc237_stone3_p12.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "defclause clause has multiple `:guard` keywords — only one `:guard` per clause is permitted (compose multiple conditions with :and)"
        )
    );
}

// ─── Probe 13 ───────────────────────────────────────────────────────────────
#[test]
fn probe_13_keyword_order_violation_rejected() {
    // Order fixed: args → :guard? → :ensure? → body. :ensure BEFORE :guard is illegal.
    let result = startup_from_file("tests/function/probe_arc237_stone3_p13.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "defclause clause has `:guard` after `:ensure` — fixed order is: args → :guard? → :ensure? → body"
        )
    );
}

// ─── Probe 14 ───────────────────────────────────────────────────────────────
#[test]
fn probe_14_complex_demo_2_2_arity_guards_plus_3_arity_ensure() {
    // Per scratch 017 ADDENDUM Demo 2 — 2 same-arity-with-different-guards
    // + 1 3-arity clause with :ensure.
    // 1 + 2 + 3 = 6; "result: sum=6"; ensure passes (starts with "result:").
    match run(":user::probe-14") {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "result: sum=6", "expected 'result: sum=6'");
        }
        other => panic!("expected Value::String('result: sum=6'); got {:?}", other),
    }
}

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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

/// Returns Ok(()) if source parses + type-checks + freezes cleanly.
fn try_startup(src: &str) -> Result<(), String> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

/// Compile + evaluate a defclause-using program. Returns the Value produced by
/// calling `:user::compute`. User source must define `:user::compute`.
fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_01_guard_true_body_fires() {
    let src = r#"
        (:wat::core::defclause :my::pick
          ([x <- :wat::core::i64] :guard (:wat::core::i64::> x 0) -> :wat::core::i64 x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::pick 42))
    "#;
    let result = run_compute(src).expect(":guard true should allow body to fire");
    assert_eq!(result, Value::i64(42));
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_guard_false_no_match_runtime_error() {
    let src = r#"
        (:wat::core::defclause :my::pick
          ([x <- :wat::core::i64] :guard (:wat::core::i64::> x 0) -> :wat::core::i64 x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::pick -5))
    "#;
    let result = run_compute(src);
    assert!(
        result.is_err(),
        ":guard false on the only clause should raise NoMatchingClause; got Ok({:?})",
        result
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_guard_false_falls_through_to_next_clause() {
    let src = r#"
        (:wat::core::defclause :my::pick
          ([x <- :wat::core::i64] :guard (:wat::core::i64::> x 100) -> :wat::core::i64 999)
          ([x <- :wat::core::i64] :guard (:wat::core::i64::> x 0) -> :wat::core::i64 x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::pick 42))
    "#;
    let result = run_compute(src).expect("second clause guard true; should fire");
    assert_eq!(
        result,
        Value::i64(42),
        "first guard (x > 100) is false for 42; second guard (x > 0) is true; second body fires"
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_factorial_demo_via_guards() {
    // Per scratch 017 ADDENDUM Demo 1 — Factorial (Erlang spirit via Path C).
    let src = r#"
        (:wat::core::defclause :my::factorial
          ([n <- :wat::core::i64] :guard (:wat::core::i64::= n 0) -> :wat::core::i64 1)
          ([n <- :wat::core::i64] :guard (:wat::core::i64::> n 0) -> :wat::core::i64
            (:wat::core::i64::* n (:my::factorial (:wat::core::i64::- n 1)))))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::factorial 5))
    "#;
    let result = run_compute(src).expect("factorial(5) should compute via guard-dispatch");
    assert_eq!(result, Value::i64(120), "5! = 120");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_guard_non_boolean_errors_at_check() {
    // :guard must produce :bool. An :i64 expression should fail type-check.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64] :guard (:wat::core::i64::+ x 1) -> :wat::core::i64 x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        ":guard returning :i64 (not :bool) should fail type-check; got Ok"
    );
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_ensure_true_returns_result() {
    let src = r#"
        (:wat::core::defclause :my::positive
          ([x <- :wat::core::i64] -> :wat::core::i64
            :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::i64::> result 0))
            x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::positive 42))
    "#;
    let result = run_compute(src).expect(":ensure true should return result");
    assert_eq!(result, Value::i64(42));
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_ensure_false_raises_postcondition() {
    let src = r#"
        (:wat::core::defclause :my::positive
          ([x <- :wat::core::i64] -> :wat::core::i64
            :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::i64::> result 0))
            x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::positive -5))
    "#;
    let result = run_compute(src);
    assert!(
        result.is_err(),
        ":ensure false (result -5 not > 0) should raise postcondition; got Ok({:?})",
        result
    );
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_ensure_fn_wrong_arity_errors_at_check() {
    // :ensure :fn must be 1-arity. 2-arity should reject at type-check.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64] -> :wat::core::i64
            :ensure (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::i64::> a b))
            x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        ":ensure :fn with arity 2 should fail type-check; got Ok"
    );
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_ensure_fn_arg_type_mismatch_errors_at_check() {
    // :ensure :fn's arg type must match the clause's declared return type.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64] -> :wat::core::i64
            :ensure (:wat::core::fn [result <- :wat::core::String] -> :wat::core::bool
                      (:wat::core::String/empty? result))
            x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        ":ensure :fn arg type :String != declared return :i64; should fail type-check; got Ok"
    );
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_ensure_fn_return_not_bool_errors_at_check() {
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64] -> :wat::core::i64
            :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::i64
                      (:wat::core::i64::+ result 1))
            x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        ":ensure :fn return :i64 (not :bool); should fail type-check; got Ok"
    );
}

// ─── Probe 11 ───────────────────────────────────────────────────────────────
#[test]
fn probe_11_full_shape_guard_and_ensure() {
    // Both :guard AND :ensure in one clause.
    let src = r#"
        (:wat::core::defclause :my::strict-positive
          ([x <- :wat::core::i64]
            :guard (:wat::core::i64::> x 0)
            :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::i64::> result 0))
            -> :wat::core::i64 x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::strict-positive 42))
    "#;
    let result = run_compute(src).expect("guard true + ensure true should return result");
    assert_eq!(result, Value::i64(42));
}

// ─── Probe 12 ───────────────────────────────────────────────────────────────
#[test]
fn probe_12_multiple_guards_rejected() {
    // ONE :guard per clause; multiple should reject.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64]
            :guard (:wat::core::i64::> x 0)
            :guard (:wat::core::i64::< x 100)
            -> :wat::core::i64 x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "multiple :guard in same clause should reject; got Ok"
    );
}

// ─── Probe 13 ───────────────────────────────────────────────────────────────
#[test]
fn probe_13_keyword_order_violation_rejected() {
    // Order fixed: args → :guard? → :ensure? → body. :ensure BEFORE :guard is illegal.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64]
            :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
                      (:wat::core::i64::> result 0))
            :guard (:wat::core::i64::> x 0)
            -> :wat::core::i64 x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        ":ensure before :guard (order violation) should reject; got Ok"
    );
}

// ─── Probe 14 ───────────────────────────────────────────────────────────────
#[test]
fn probe_14_complex_demo_2_2_arity_guards_plus_3_arity_ensure() {
    // Per scratch 017 ADDENDUM Demo 2 — 2 same-arity-with-different-guards
    // + 1 3-arity clause with :ensure.
    let src = r#"
        (:wat::core::defclause :my::process
          ;; 2-arity clause 1: guard x > y
          ([x <- :wat::core::i64 y <- :wat::core::i64]
            :guard (:wat::core::i64::> x y)
            -> :wat::core::String
            (:wat::core::String/concat "x>y:" (:wat::core::i64/to-string x)))
          ;; 2-arity clause 2: guard x < y
          ([x <- :wat::core::i64 y <- :wat::core::i64]
            :guard (:wat::core::i64::< x y)
            -> :wat::core::String
            (:wat::core::String/concat "x<y:" (:wat::core::i64/to-string y)))
          ;; 3-arity clause: ensure result starts with "result:"
          ([x <- :wat::core::i64 y <- :wat::core::i64 z <- :wat::core::i64]
            :ensure (:wat::core::fn [result <- :wat::core::String] -> :wat::core::bool
                      (:wat::core::String/starts-with? result "result:"))
            -> :wat::core::String
            (:wat::core::String/concat "result: sum="
              (:wat::core::i64/to-string
                (:wat::core::i64::+ (:wat::core::i64::+ x y) z)))))
        (:wat::core::defn :user::compute [] -> :wat::core::String (:my::process 1 2 3))
    "#;
    let result = run_compute(src).expect("3-arity clause with ensure should compute + validate");
    // 1 + 2 + 3 = 6; "result: sum=6"; ensure passes (starts with "result:")
    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "result: sum=6", "expected 'result: sum=6'");
        }
        other => panic!("expected Value::String('result: sum=6'); got {:?}", other),
    }
}

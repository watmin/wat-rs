//! FM 2-bis probe — arc 237 Stone 237.2: `:wat::core::defclause` substrate mint.
//!
//! Verifies the load-bearing contract: NEW `Value::wat__core__clauses` variant +
//! parser + per-clause type-check + arity-match dispatch + per-clause return types.
//! Consumes Stone 237.1's typeunion + bounded-existential unify for typeunion-typed
//! args.
//!
//! Doctrine (per docs/arc/2026/05/237-polymorphism-consolidation/):
//!   - defclause is THE multi-arity entity kind (defn stays single-arity)
//!   - First-match-wins dispatch (arity match → arg type match → fire body)
//!   - Per-clause return types (each clause declares its own -> :T; OR Option A
//!     top-level shared return)
//!   - Vector-literal args + List-form clauses per feedback_clojure_not_scheme
//!   - No :guard / :ensure yet (Stone 237.3); no rich :NoMatchingClause yet (237.4);
//!     no variadic rest (237.5)
//!
//! Probe contracts (12):
//!   1.  Single-clause defclause parses + type-checks (basic foundation)
//!   2.  Multi-arity dispatches by arity at call site
//!   3.  Same-arity multi-clause dispatches by arg type (typeunion-driven)
//!   4.  typeunion-typed arg accepts via bounded existential (Stone 237.1 integration)
//!   5.  Option A: top-level shared return type applies to all clauses
//!   6.  Option B: per-clause return types; caller picks via clause match
//!   7.  Body return-type mismatch errors at check time
//!   8.  No matching clause at call site errors at check time
//!   9.  Runtime computed result correct: (defclause :add ([x <- :i64 y <- :i64] -> :i64 ...))
//!   10. Single-clause defclause behaves like defn (executable)
//!   11. Empty defclause (0 clauses) rejected at parse time
//!   12. Binding contract preserved: [name <- :Type] only; literal patterns not allowed
//!
//! Initial state: file does not compile — Value::wat__core__clauses doesn't exist,
//! parser doesn't recognize defclause, type-checker/evaluator don't dispatch.
//!
//! Post-stone 237.2: 12/12 PASS.
//!
//! Per FM 2-bis (recovery doc § 6): probe COMMITTED before BRIEF; BRIEF cites
//! this file verbatim as "the working contract sonnet must satisfy."

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────────

/// Returns Ok(()) if source parses + type-checks + freezes cleanly.
/// Returns Err(message) on any failure (parse / check / startup).
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
fn probe_01_single_clause_defclause_basic() {
    // Foundation: defclause with ONE clause parses + type-checks.
    // Functionally equivalent to defn but with the new form shape.
    let src = r#"
        (:wat::core::defclause :my::identity
          ([x <- :wat::core::i64] -> :wat::core::i64 x))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    try_startup(src).expect("single-clause defclause should parse + type-check");
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_multi_arity_dispatches_by_arity() {
    // Two clauses with different arities. Call site picks by arg-count.
    let src = r#"
        (:wat::core::defclause :my::add
          ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+ x y))
          ([x <- :wat::core::i64 y <- :wat::core::i64 z <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+ (:wat::core::i64::+ x y) z)))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::add 10 20 30))
    "#;
    let result = run_compute(src).expect("3-arity defclause call should evaluate");
    assert_eq!(
        result,
        Value::i64(60),
        "3-arity clause fires; returns 10 + 20 + 30 = 60"
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_same_arity_different_types_dispatches_by_type() {
    // Two same-arity clauses with different arg types. Dispatch picks by type match.
    let src = r#"
        (:wat::core::defclause :my::sum
          ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+ x y))
          ([x <- :wat::core::f64 y <- :wat::core::f64] -> :wat::core::f64
            (:wat::core::f64::+ x y)))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::sum 7 3))
    "#;
    let result = run_compute(src).expect("i64 clause should fire for i64 args");
    assert_eq!(
        result,
        Value::i64(10),
        "(:my::sum 7 3) → first i64+i64 clause → 7+3 = 10"
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_typeunion_arg_accepts_via_bounded_existential() {
    // Stone 237.1 typeunion + Stone 237.2 defclause integration.
    // [x <- :Numeric] accepts both :i64 and :f64 (members of :Numeric).
    let src = r#"
        (:wat::core::typeunion :my::Numeric [:wat::core::i64 :wat::core::f64])
        (:wat::core::defclause :my::identity-num
          ([x <- :my::Numeric] -> :my::Numeric x))
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::do
                      (:my::identity-num 42)
                      (:my::identity-num 3.14)
                      nil))
    "#;
    try_startup(src)
        .expect("typeunion-typed defclause arg should accept i64 + f64 via bounded existential");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_shared_return_type_applies_to_all_clauses() {
    // Option A: top-level :T after :name; all clauses must return :T.
    let src = r#"
        (:wat::core::defclause :my::pick -> :wat::core::i64
          ([x <- :wat::core::i64] x)
          ([x <- :wat::core::i64 y <- :wat::core::i64] (:wat::core::i64::+ x y)))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::pick 5 7))
    "#;
    let result = run_compute(src).expect("shared return :i64 should accept i64 clauses");
    assert_eq!(result, Value::i64(12), "2-arity clause fires; 5+7=12");
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_per_clause_return_types_pick_at_call_site() {
    // Option B: each clause declares its OWN return type.
    // Caller's inferred type = the matching clause's return type.
    let src = r#"
        (:wat::core::defclause :my::process
          ([x <- :wat::core::i64] -> :wat::core::i64 x)
          ([x <- :wat::core::f64] -> :wat::core::f64 x))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::process 42))
    "#;
    let result = run_compute(src).expect("i64 clause fires; return :i64 matches :user::compute return");
    assert_eq!(result, Value::i64(42));
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_body_return_type_mismatch_errors() {
    // Clause body returns f64 but declares -> :i64. Should fail at type-check.
    let src = r#"
        (:wat::core::defclause :my::bad
          ([x <- :wat::core::i64] -> :wat::core::i64 3.14))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "body returning :f64 with declared -> :i64 should fail type-check; got Ok"
    );
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_no_matching_clause_at_call_site_errors() {
    // Call with arg types that no clause accepts.
    let src = r#"
        (:wat::core::defclause :my::only-i64
          ([x <- :wat::core::i64] -> :wat::core::i64 x))
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::do
                      (:my::only-i64 "string-arg")
                      nil))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "calling :i64-typed clause with :String arg should fail at type-check"
    );
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_runtime_computes_correct_result() {
    // End-to-end runtime check: defclause + arithmetic produces correct Value.
    let src = r#"
        (:wat::core::defclause :my::factorial-like
          ([n <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::* n n)))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::factorial-like 7))
    "#;
    let result = run_compute(src).expect("defclause should evaluate to i64 result");
    assert_eq!(result, Value::i64(49), "7*7=49");
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_single_clause_defclause_equivalent_to_defn() {
    // A 1-clause defclause should be functionally equivalent to a defn.
    let src = r#"
        (:wat::core::defclause :my::double
          ([n <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::* n 2)))
        (:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::double 21))
    "#;
    let result = run_compute(src).expect("single-clause defclause runs like defn");
    assert_eq!(result, Value::i64(42));
}

// ─── Probe 11 ───────────────────────────────────────────────────────────────
#[test]
fn probe_11_empty_defclause_rejected() {
    // defclause with ZERO clauses should be rejected at parse/registration.
    let src = r#"
        (:wat::core::defclause :my::empty)
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "defclause with 0 clauses should be rejected; got Ok"
    );
}

// ─── Probe 12 ───────────────────────────────────────────────────────────────
#[test]
fn probe_12_binding_contract_preserved_no_literal_patterns() {
    // Per arc 159/169/234 + Path C lock: clause args MUST be [name <- :Type].
    // Literal patterns (e.g., [0 <- :i64]) are NOT a valid arg shape.
    // Sonnet should reject this at parse time.
    let src = r#"
        (:wat::core::defclause :my::bad-pattern
          ([0 <- :wat::core::i64] -> :wat::core::i64 1))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "literal-pattern arg [0 <- :i64] should be rejected per binding contract; got Ok"
    );
}

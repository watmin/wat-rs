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

//! Wat source: tests/function/probe_arc237_stone2_defclause_substrate.wat
//! Negative fixtures: probe_arc237_stone2_p07.wat.bad, probe_arc237_stone2_p08.wat.bad,
//!   probe_arc237_stone2_p11.wat.bad, probe_arc237_stone2_p12.wat.bad.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeErrorKind, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for stone2 defclause-substrate fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_01_single_clause_defclause_basic() {
    // Foundation: defclause with ONE clause parses + type-checks.
    // Functionally equivalent to defn but with the new form shape.
    // The main fixture compiles only if all defclauses are valid — probe 1's
    // `:p01::identity` is the single-clause baseline that must parse + type-check.
    startup_beside(file!()).expect("single-clause defclause should parse + type-check");
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_02_multi_arity_dispatches_by_arity() {
    // Two clauses with different arities. Call site picks by arg-count.
    // 3-arity clause fires; returns 10 + 20 + 30 = 60.
    assert_eq!(run(":user::probe-02"), Value::i64(60), "3-arity clause fires; 10+20+30 = 60");
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_03_same_arity_different_types_dispatches_by_type() {
    // Two same-arity clauses with different arg types. Dispatch picks by type match.
    // i64 args → i64 clause fires; 7+3 = 10.
    assert_eq!(
        run(":user::probe-03"),
        Value::i64(10),
        "(:p03::sum 7 3) → first i64+i64 clause → 7+3 = 10",
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_04_typeunion_arg_accepts_via_bounded_existential() {
    // Stone 237.1 typeunion + Stone 237.2 defclause integration.
    // [x <- :Numeric] accepts both :i64 and :f64 (members of :Numeric).
    // This is a TYPE-CHECKING assertion: the whole fixture (including probe-04's
    // body that calls identity-num with both 42 and 3.14) must compile cleanly.
    // Startup success = bounded-existential unifier accepted both types at check.
    startup_beside(file!())
        .expect("typeunion-typed defclause arg should accept i64 + f64 via bounded existential");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_05_shared_return_type_applies_to_all_clauses() {
    // Option A: top-level :T after :name; all clauses must return :T.
    // 2-arity clause fires; 5+7=12.
    assert_eq!(run(":user::probe-05"), Value::i64(12), "2-arity clause fires; 5+7=12");
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_06_per_clause_return_types_pick_at_call_site() {
    // Option B: each clause declares its OWN return type.
    // Caller's inferred type = the matching clause's return type.
    // i64 arg → i64 clause fires → i64(42).
    assert_eq!(run(":user::probe-06"), Value::i64(42));
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
#[test]
fn probe_07_body_return_type_mismatch_errors() {
    // Clause body returns f64 but declares -> :i64. Should fail at type-check.
    let result = startup_from_file("tests/function/probe_arc237_stone2_p07.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":my::bad/clause#1"
            && expected == ":wat::core::i64"
            && got == ":wat::core::f64"
    );
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
#[test]
fn probe_08_no_matching_clause_at_call_site_errors() {
    // Call with arg types that no clause accepts.
    let result = startup_from_file("tests/function/probe_arc237_stone2_p08.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, .. }
            if name == ":my::only-i64"
            && *called_arity == 1
            && called_arg_types.as_slice() == [":wat::core::String".to_string()]
    );
}

// ─── Probe 9 ────────────────────────────────────────────────────────────────
#[test]
fn probe_09_runtime_computes_correct_result() {
    // End-to-end runtime check: defclause + arithmetic produces correct Value.
    // n*n → 7*7 = 49.
    assert_eq!(run(":user::probe-09"), Value::i64(49), "7*7=49");
}

// ─── Probe 10 ───────────────────────────────────────────────────────────────
#[test]
fn probe_10_single_clause_defclause_equivalent_to_defn() {
    // A 1-clause defclause should be functionally equivalent to a defn.
    // n*2 → 21*2 = 42.
    assert_eq!(run(":user::probe-10"), Value::i64(42));
}

// ─── Probe 11 ───────────────────────────────────────────────────────────────
#[test]
fn probe_11_empty_defclause_rejected() {
    // defclause with ZERO clauses should be rejected at parse/registration.
    let result = startup_from_file("tests/function/probe_arc237_stone2_p11.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "defclause must have at least one clause"
        )
    );
}

// ─── Probe 12 ───────────────────────────────────────────────────────────────
#[test]
fn probe_12_binding_contract_preserved_no_literal_patterns() {
    // Per arc 159/169/234 + Path C lock: clause args MUST be [name <- :Type].
    // Literal patterns (e.g., [0 <- :i64]) are NOT a valid arg shape.
    // Sonnet should reject this at parse time.
    let result = startup_from_file("tests/function/probe_arc237_stone2_p12.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Runtime(e) if matches!(
            e.kind(),
            RuntimeErrorKind::MalformedForm { head, reason }
                if head == ":wat::core::defclause"
                && reason == "name must be a plain symbol (not a keyword, literal, or nested form)"
        )
    );
}

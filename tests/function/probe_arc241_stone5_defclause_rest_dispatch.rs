//! FM 2-bis probe for Stone 241.5 — defclause `&` rest-binder runtime dispatch.
//!
//! ## Why this probe
//!
//! Stone 241.4 settled the storage foundation:
//!   - Canonical parser parses `& name <- :T` when `allow_rest_binder: true`
//!   - A4 (parse_defclause_args inlined) sets allow_rest_binder: true
//!   - Clause struct gained `rest_param: Option<(String, TypeExpr)>`
//!   - Parser threads it through to clause storage
//!
//! Stone 241.5 wires the dispatch: `eval_call_to_defclause_with_vals` at
//! `src/runtime.rs:7198` consumes `Clause.rest_param` for:
//!   1. Variadic-min arity match (called_arity >= fixed_arity when rest exists)
//!   2. Element-type check per rest value (against Vector<T>'s T)
//!   3. Rest values collected into Value::Vector
//!   4. Bound at rest_param.name in the clause scope
//!
//! ## What this probe proves
//!
//! Pre-stone (HEAD `cfe93a22`+): contracts that exercise rest-binder DISPATCH
//! fail because the substrate's dispatcher uses strict arity equality. The
//! 237.8b Gate 1 (`gate_1_defclause_supports_rest_binder`) is currently
//! `#[ignore]`'d with this stone as its named follow-up.
//!
//! Post-stone: contracts pass; Gate 1 un-ignored and PASSING.
//!
//! ## FM 2-bis nature: BEHAVIORAL EXTENSION probe
//!
//! Stone 241.4 added storage (Clause.rest_param); Stone 241.5 adds the
//! behavior that consumes it. This probe disconfirms the missing behavior at
//! HEAD and confirms it post-stone.
//!
//! Run: `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch`

//! Wat source: tests/function/probe_arc241_stone5_defclause_rest_dispatch.wat
//! Negative fixtures: probe_arc241_stone5_c05.wat.bad, probe_arc241_stone5_c06.wat.bad,
//!   probe_arc241_stone5_c07.wat.bad.

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, ClauseFailureReason, RuntimeErrorKind, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for stone5 rest-dispatch fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ─── Contracts 1–4: rest-binder dispatch success paths ───────────────────────

#[test]
fn contract_01_variadic_min_with_rest_succeeds() {
    // defclause with [fixed & rest <- :Vector<:i64>]; called with fixed + N rest values.
    // Rest collected into Vector; foldl folds them; 1+2+3+4 = 10.
    assert_eq!(
        run(":user::c01-variadic"),
        Value::i64(10),
        "1+2+3+4 = 10 via & rest-binder fold",
    );
}

#[test]
fn contract_02_empty_rest_succeeds() {
    // Called with exactly fixed-arity values; rest is empty Vector; fold returns seed 42.
    assert_eq!(
        run(":user::c02-empty-rest"),
        Value::i64(42),
        "fold of empty rest with seed 42 returns 42",
    );
}

#[test]
fn contract_03_rest_only_succeeds() {
    // Rest-only clause (no fixed args before `&`); 3 args → length 3.
    assert_eq!(
        run(":user::c03-rest-only"),
        Value::i64(3),
        "3 args collected into rest Vector; length is 3",
    );
}

#[test]
fn contract_04_rest_only_empty_call_succeeds() {
    // Rest-only clause called with ZERO args; rest is empty Vector; length is 0.
    assert_eq!(
        run(":user::c04-rest-only-empty"),
        Value::i64(0),
        "0 args → empty rest Vector; length is 0",
    );
}

// ─── Contracts 5–7: error paths ──────────────────────────────────────────────

#[test]
fn contract_05_rest_element_type_mismatch_errors() {
    // Passing "three" (String) where Vector<i64> element is expected.
    // Type mismatch is caught at eval time (rest element types are checked at dispatch).
    let world = startup_from_file("tests/function/probe_arc241_stone5_c05.wat.bad")
        .expect("startup should succeed (rest element type mismatch caught at dispatch, not check)");
    let func = world.symbols().get(":user::bad").expect(":user::bad").clone();
    let result = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!());
    assert!(
        matches!(
            &result,
            Err(e) if matches!(
                e.kind(),
                RuntimeErrorKind::NoMatchingClause { name, called_arity, attempted_clauses, .. }
                    if name == ":my::sum-all"
                    && *called_arity == 3
                    && attempted_clauses.len() == 1
                    && matches!(
                        &attempted_clauses[0].failure_reason,
                        ClauseFailureReason::ArgTypeMismatch { position, expected, got }
                            if *position == 2 && expected == ":wat::core::i64" && got == ":wat::core::String"
                    )
            )
        ),
        "rest element type mismatch must error at eval/dispatch with RuntimeErrorKind::NoMatchingClause{{name: \":my::sum-all\", ArgTypeMismatch pos 2}}; got {:?}",
        result
    );
}

#[test]
fn contract_06_under_supply_below_fixed_errors() {
    // Clause has 2 fixed args + rest; calling with only 1 arg must error. startup MUST fail.
    let result = startup_from_file("tests/function/probe_arc241_stone5_c06.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, .. }
            if name == ":my::pair"
            && *called_arity == 1
            && called_arg_types.as_slice() == [":wat::core::i64".to_string()]
    );
}

#[test]
fn contract_07_fixed_only_strict_arity_preserved() {
    // Clause WITHOUT rest_param. Called with extra args → strict arity rejection.
    // Stone 241.5's variadic-min behavior MUST NOT apply when rest_param is None.
    let result = startup_from_file("tests/function/probe_arc241_stone5_c07.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::NoMatchingClauseAtCallSite { name, called_arity, called_arg_types, .. }
            if name == ":my::strict"
            && *called_arity == 2
            && called_arg_types.as_slice() == [":wat::core::i64".to_string(), ":wat::core::i64".to_string()]
    );
}

// ─── Contract 8: regression on mixed dispatch ────────────────────────────────

#[test]
fn contract_08_mixed_clause_set_first_match_wins() {
    // First clause = fixed [x <- :i64]; second clause = [x <- :i64 & rest <- :Vector<:i64>].
    // (10 20 30) → first clause arity-mismatches; second clause matches; 10+20+30 = 60.
    assert_eq!(
        run(":user::c08-mixed"),
        Value::i64(60),
        "10+20+30 = 60 via second (rest-binder) clause; first clause arity-mismatched",
    );
}

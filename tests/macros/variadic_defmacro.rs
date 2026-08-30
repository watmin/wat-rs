//! End-to-end tests for variadic `:wat::core::defmacro` — the `&`
//! rest-param syntax. Fixed params bind positionally; args after the
//! `&` rest-binder collect into a `WatAST::List` that a `,@name`
//! unquote-splicing drops into the template.
//!
//! This is the last macro-system prereq for the `pipeline` one-liner
//! composer (arc 004 item #5). With variadic, `(pipeline source
//! stage1 stage2 stage3 sink)` becomes expressible as a single macro
//! whose rest-param captures every stage and emits a let chain.
//!
//! Coverage:
//!
//! - User-defined variadic macro that splices its rest-args into a
//!   core form (the canonical use).
//! - Zero rest-args — empty splice still produces a well-formed AST.
//! - Fixed-param-then-rest ordering.
//! - Arity error: fewer than fixed_arity args.
//! - Multiple `&` markers rejected at registration.
//! - Rest-param without a following binder rejected.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to named fns; canonical nil main appended.
//!
//! Positive tests use the co-located fixture: variadic_defmacro.wat
//! Negative tests use explicit bad fixtures in tests/macros/:
//!   variadic_defmacro_bad_arity.wat
//!   variadic_defmacro_bad_double_rest.wat
//!   variadic_defmacro_bad_rest_no_binder.wat

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

// just-eval (rubric): each probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.

// ─── Canonical use: splice into a core form ───────────────────────────

#[test]
fn variadic_macro_splices_rest_into_vec_ctor() {
    // `(my::vec-of :wat::core::i64 1 2 3)` expands to
    // `(:wat::core::Vector :- [:wat::core::i64] 1 2 3)`. The `& (items ...)` rest-binder
    // collects the trailing 1 2 3 into a list; `,@items` splices them.
    let got = call_beside_value(file!(), ":my::compute-splice").expect("compute should run");
    assert!(matches!(got, Value::i64(10)));
}

// ─── Zero rest-args ───────────────────────────────────────────────────

#[test]
fn variadic_macro_with_zero_rest_args_produces_empty_splice() {
    match call_beside_value(file!(), ":my::compute-empty").expect("compute should run") {
        Value::Vec(items) => assert_eq!(items.len(), 0),
        other => panic!("expected empty Vec; got {:?}", other),
    }
}

// ─── Fixed params + rest combined ─────────────────────────────────────

#[test]
fn variadic_macro_mixes_fixed_params_and_rest() {
    // Simpler shape: macro expands to `(vec :wat::core::i64 init ,@items)` and
    // we sum-fold the result. Keeps the splice the point of the test.
    let got = call_beside_value(file!(), ":my::compute-sum").expect("compute should run");
    assert!(matches!(got, Value::i64(106)));
}

// ─── Arity error: too few args ────────────────────────────────────────

#[test]
fn variadic_macro_requires_at_least_fixed_arity() {
    // `(my::sum-of)` with NO args — fixed-arity of :init is 1, so
    // zero args is a short call. Surfaces as a macro-expansion
    // ArityMismatch during startup.
    match startup_from_file("tests/macros/variadic_defmacro_bad_arity.wat") {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail"),
    }
}

// ─── Malformed rest-marker ────────────────────────────────────────────

#[test]
fn double_rest_marker_refused_at_registration() {
    match startup_from_file("tests/macros/variadic_defmacro_bad_double_rest.wat") {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on duplicate `&`"),
    }
}

#[test]
fn rest_marker_without_binder_refused_at_registration() {
    match startup_from_file("tests/macros/variadic_defmacro_bad_rest_no_binder.wat") {
        Err(StartupError::Macro(_)) => {}
        Err(other) => panic!("expected Macro error; got {:?}", other),
        Ok(_) => panic!("expected startup to fail on `&` without binder"),
    }
}

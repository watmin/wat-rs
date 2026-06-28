//! Regression guard for `:wat::core::apply` (arc 232 Stone 232.0).
//!
//! The original 3 probes (commit `5c7dddf`) confirmed that dynamic
//! keyword-as-head invocation was NOT supported by the substrate: binding
//! a keyword to a local and calling via the binding raised
//! `NotCallable { got: "wat::core::keyword" }`. That finding drove the
//! design and implementation of `:wat::core::apply`.
//!
//! After Stone 232.0 ships, the 3 existing probes are REWRITTEN to use
//! the new primitive — they become the load-bearing regression guard that
//! the substrate gap cannot reopen. Five new probes cover Clojure-shape
//! contract edge cases.
//!
//! Probe inventory:
//!   1. Bound substrate-verb keyword dispatched via apply
//!   2. Runtime-built keyword dispatched via apply
//!   3. Mangled-namespace user defn dispatched via apply
//!   4. Leading positional args + tail vector (mixed spread shape)
//!   5. Empty tail vector (spread vec is [])
//!   6. Special-form head rejection (:wat::core::defn → error)
//!   7. Non-keyword head rejection (String → type error)
//!   8. Non-vector last arg rejection (trailing i64 instead of Vector)

//! Wat source: tests/function/probe_diagnostic_dynamic_keyword_invocation.wat
//! Negative fixtures: probe_diagnostic_non_keyword_bad.wat (probe 7),
//!   probe_diagnostic_non_vector_bad.wat (probe 8).
//! Runtime-fail fn in main fixture: :user::probe-6-err (probe 6).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for diagnostic apply fixture");
    let ast = wat::parse_one!(&format!("({fn_name})")).expect("parse named fn call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

// ─── Probe 1 (rewritten) ────────────────────────────────────────────────────
//
// Original: bound substrate-verb keyword as head → FAIL NotCallable.
// Rewritten: use (:wat::core::apply -> :wat::core::i64 plus [2 3]) → PASS.
// Arc 009 lifts a registered keyword to a fn value; apply also accepts fn
// values as head so both keyword and fn-valued bindings dispatch correctly.
#[test]
fn probe_1_bound_keyword_invokes_substrate_verb() {
    assert_eq!(run(":user::probe-1"), Value::i64(5), "apply of bound-keyword plus [2 3] → 5");
}

// ─── Probe 2 (rewritten) ────────────────────────────────────────────────────
//
// Original: runtime-built keyword as head → FAIL NotCallable.
// Rewritten: use (:wat::core::apply -> :T plus [2 3]) → PASS.
// keyword/from-string builds a Value::keyword (never lifted to fn);
// eval_apply accepts keyword values directly via the substrate-impl path.
#[test]
fn probe_2_runtime_built_keyword_invokes_substrate_verb() {
    assert_eq!(run(":user::probe-2"), Value::i64(5), "apply of runtime-built keyword plus [2 3] → 5");
}

// ─── Probe 3 (rewritten) ────────────────────────────────────────────────────
//
// Original: mangled-namespace user defn as head → FAIL NotCallable.
// Rewritten: use (:wat::core::apply -> :T verb ["world"]) → PASS.
// Mirrors defprotocol's dispatch pattern: build FQDN keyword at runtime +
// invoke via apply. keyword/from-string returns a raw keyword value
// (NOT lifted to fn) so eval_apply dispatches via sym.functions.
#[test]
fn probe_3_mangled_namespace_invokes_user_defn() {
    match run(":user::probe-3") {
        Value::String(s) => assert_eq!(s.as_ref(), "hello world", "apply of mangled-namespace verb → 'hello world'"),
        other => panic!("probe 3: expected Value::String('hello world'); got {:?}", other),
    }
}

// ─── Probe 4 (new) ──────────────────────────────────────────────────────────
//
// Leading positional args + tail spread vector.
// (:wat::core::apply -> :i64 :ns::add4 1 2 [3 4]) → 10
// The head :ns::add4 is a literal keyword; Arc 009 lifts it to fn value.
// eval_apply handles fn-valued head directly.
#[test]
fn probe_4_apply_with_leading_args_and_tail_vec() {
    assert_eq!(run(":user::probe-4"), Value::i64(10), "apply with leading args + tail vec: 1+2+3+4 = 10");
}

// ─── Probe 5 (new) ──────────────────────────────────────────────────────────
//
// Empty tail vector — spread contributes zero args.
// (:wat::core::apply -> :String :ns::greet []) → "hello"
// :ns::greet literal keyword lifts to fn via Arc 009; apply handles fn head.
#[test]
fn probe_5_apply_with_empty_args_vec() {
    match run(":user::probe-5") {
        Value::String(s) => assert_eq!(s.as_ref(), "hello", "apply with empty tail vec → 'hello'"),
        other => panic!("probe 5: expected Value::String('hello'); got {:?}", other),
    }
}

// ─── Probe 6 (new) ──────────────────────────────────────────────────────────
//
// Special-form head rejection. apply cannot dispatch to declaration / language
// forms; it must error with a clear diagnostic (STOP-8 guard).
// :wat::core::defn is a declaration form — apply rejects it at RUNTIME
// (the keyword is built dynamically; type-checker can't know "defn" is special
// at compile time).
#[test]
fn probe_6_apply_rejects_special_form_head() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::probe-6-err)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_err(),
        "apply of special-form head (:defn) must error at runtime; got Ok",
    );
}

// ─── Probe 7 (new) ──────────────────────────────────────────────────────────
//
// Non-keyword head rejection. If head evaluates to something other than a
// keyword or fn, apply must reject with a type error.
// The error occurs at eval time (type checker can't statically reject a
// String-literal head in apply — apply head is dispatched dynamically).
#[test]
fn probe_7_apply_rejects_non_keyword_head() {
    let world = startup_from_file("tests/function/probe_diagnostic_non_keyword_bad.wat")
        .expect("startup should succeed (non-keyword head in apply caught at eval, not check)");
    let ast = wat::parse_one!("(:user::bad)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(result.is_err(), "non-keyword head (String) must error at eval; got Ok");
}

// ─── Probe 8 (new) ──────────────────────────────────────────────────────────
//
// Non-vector last arg rejection. The trailing spread arg MUST be a
// :wat::core::Vector; passing a plain i64 must produce an error.
// Error occurs at eval time (apply's spread-arg check is dynamic).
#[test]
fn probe_8_apply_rejects_non_vector_last_arg() {
    let world = startup_from_file("tests/function/probe_diagnostic_non_vector_bad.wat")
        .expect("startup should succeed (non-vector spread arg caught at eval, not check)");
    let ast = wat::parse_one!("(:user::bad)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(result.is_err(), "non-vector spread arg (i64) must error at eval; got Ok");
}

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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

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

// ─── Probe 1 (rewritten) ────────────────────────────────────────────────────
//
// Original: bound substrate-verb keyword as head → FAIL NotCallable.
// Rewritten: use (:wat::core::apply -> :wat::core::i64 plus [2 3]) → PASS.
// Arc 009 lifts a registered keyword to a fn value; apply also accepts fn
// values as head so both keyword and fn-valued bindings dispatch correctly.
#[test]
fn probe_1_bound_keyword_invokes_substrate_verb() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [plus :wat::core::i64::+]
      (:wat::core::apply -> :wat::core::i64 plus [2 3])))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 1 result: {}", s);
            assert!(
                s.contains("5"),
                "Probe 1: apply of bound-keyword produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 (rewritten) ────────────────────────────────────────────────────
//
// Original: runtime-built keyword as head → FAIL NotCallable.
// Rewritten: use (:wat::core::apply -> :T plus [2 3]) → PASS.
// keyword/from-string builds a Value::keyword (never lifted to fn);
// eval_apply accepts keyword values directly via the substrate-impl path.
#[test]
fn probe_2_runtime_built_keyword_invokes_substrate_verb() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [plus (:wat::core::keyword/from-string "wat::core::i64::+")]
      (:wat::core::apply -> :wat::core::i64 plus [2 3])))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert!(
                s.contains("5"),
                "Probe 2: apply of runtime-built keyword produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
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
    let src = r#"
(:wat::core::defn :ns::greeting [name <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat "hello " name))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
      [verb (:wat::core::keyword/from-string "ns::greeting")]
      (:wat::core::apply -> :wat::core::String verb ["world"])))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 3 result: {}", s);
            assert!(
                s.contains("hello world"),
                "Probe 3: apply of mangled-namespace user defn produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 3 FAILED: {}", e),
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
    let src = r#"
(:wat::core::defn :ns::add4 [a <- :wat::core::i64 b <- :wat::core::i64 c <- :wat::core::i64 d <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::do
      (:wat::core::i64::+
        (:wat::core::i64::+ a b)
        (:wat::core::i64::+ c d))))

(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::apply -> :wat::core::i64 :ns::add4 1 2 [3 4]))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 4 result: {}", s);
            assert!(
                s.contains("10"),
                "Probe 4: apply with leading args + tail vec produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 (new) ──────────────────────────────────────────────────────────
//
// Empty tail vector — spread contributes zero args.
// (:wat::core::apply -> :String :ns::greet []) → "hello"
// :ns::greet literal keyword lifts to fn via Arc 009; apply handles fn head.
#[test]
fn probe_5_apply_with_empty_args_vec() {
    let src = r#"
(:wat::core::defn :ns::greet [] -> :wat::core::String "hello")

(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::apply -> :wat::core::String :ns::greet []))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 5 result: {}", s);
            assert!(
                s.contains("hello"),
                "Probe 5: apply with empty tail vec produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 (new) ──────────────────────────────────────────────────────────
//
// Special-form head rejection. apply cannot dispatch to declaration / language
// forms; it must error with a clear diagnostic (STOP-8 guard).
// :wat::core::defn is a declaration form — apply rejects it.
#[test]
fn probe_6_apply_rejects_special_form_head() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::apply -> :wat::core::String (:wat::core::keyword/from-string "wat::core::defn") []))
"#;
    match run_compute(src) {
        Ok(v) => panic!(
            "Probe 6: expected error for special-form head; got {:?}",
            v
        ),
        Err(e) => {
            println!("Probe 6 error (expected): {}", e);
            assert!(
                e.contains("apply") || e.contains("special form") || e.contains("defn"),
                "Probe 6: error message doesn't mention apply or special form: {}",
                e
            );
        }
    }
}

// ─── Probe 7 (new) ──────────────────────────────────────────────────────────
//
// Non-keyword head rejection. If head evaluates to something other than a
// keyword or fn, apply must reject with a type error.
#[test]
fn probe_7_apply_rejects_non_keyword_head() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::String (:wat::core::apply -> :wat::core::String "not-a-keyword" []))
"#;
    match run_compute(src) {
        Ok(v) => panic!(
            "Probe 7: expected error for non-keyword head; got {:?}",
            v
        ),
        Err(e) => {
            println!("Probe 7 error (expected): {}", e);
            assert!(
                e.contains("keyword") || e.contains("String") || e.contains("apply"),
                "Probe 7: error message doesn't mention the type mismatch: {}",
                e
            );
        }
    }
}

// ─── Probe 8 (new) ──────────────────────────────────────────────────────────
//
// Non-vector last arg rejection. The trailing spread arg MUST be a
// :wat::core::Vector; passing a plain i64 must produce an error.
#[test]
fn probe_8_apply_rejects_non_vector_last_arg() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::apply -> :wat::core::i64 (:wat::core::keyword/from-string "wat::core::i64::+") 42))
"#;
    match run_compute(src) {
        Ok(v) => panic!(
            "Probe 8: expected error for non-vector spread arg; got {:?}",
            v
        ),
        Err(e) => {
            println!("Probe 8 error (expected): {}", e);
            assert!(
                e.contains("Vector") || e.contains("i64") || e.contains("apply"),
                "Probe 8: error message doesn't describe the spread-arg type mismatch: {}",
                e
            );
        }
    }
}

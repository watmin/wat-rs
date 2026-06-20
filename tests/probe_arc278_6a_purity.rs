//! Arc 278 — Stone 6a: purity inference (`:wat::rete::pure?`), the shared fence for the capability tier.
//! RED at HEAD (`:wat::rete::pure?` has no dispatch arm → eval error). GREEN when 6a lands.
//! Contract: DESIGN-STONE-6a-purity-inference.md.
//!
//! The fence is DEFAULT-DENY: a head is pure only if proven pure (a known-pure intrinsic, or a user fn
//! whose body is transitively pure); anything else is rejected. "Pure" = a DETERMINISTIC function of the
//! facts — impurity is either an effect (`is_effectful_op` namespaces) OR non-determinism (randomness).
//! Load-bearing: #4 (`Uuid/v4` random → false, the case default-allow gets wrong), #4b (`Uuid/v5`
//! deterministic → true, the boundary), #6 (a user fn transitively calling an impure op → false).
//!
//! Run: cargo test --release -p wat --test probe_arc278_6a_purity

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::core::defn :test::pure-double [n <- :wat::core::i64] -> :wat::core::i64\n\
  (:wat::core::* n 2))\n\
\n\
(:wat::core::defn :test::impure-uuid [] -> :wat::core::Uuid\n\
  (:wat::core::Uuid/v4))\n\
\n\
(:wat::core::defn :test::countdown [n <- :wat::core::i64] -> :wat::core::i64\n\
  (:wat::core::if (:wat::core::<= n 0)\n\
    0\n\
    (:test::countdown (:wat::core::- n 1))))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Classify `expr` (a wat form, as source text) by asking `(:wat::rete::pure? (quote <expr>))`.
fn pure(expr: &str) -> Value {
    let compute = format!("(:wat::rete::pure? (:wat::core::quote {expr}))");
    let world = startup_from_source(WORLD, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!(&compute).expect("parse compute");
    eval_in_frozen(&ast, &world, &Environment::new()).expect("pure? should run").value_owned()
}

fn is_true(v: &Value) -> bool { matches!(v, Value::bool(true)) }
fn is_false(v: &Value) -> bool { matches!(v, Value::bool(false)) }

/// 1 — pure intrinsic arithmetic/comparison expr is pure.
#[test]
fn pure_intrinsic_expr() {
    let v = pure("(:wat::core::> (:wat::core::- 5 3) 1)");
    assert!(is_true(&v), "pure arithmetic/comparison → true; got {v:?}");
}

/// 2 — pure string predicate is pure.
#[test]
fn pure_string_predicate() {
    let v = pure("(:wat::core::string::starts-with? \"abc\" \"a\")");
    assert!(is_true(&v), "string::starts-with? → true; got {v:?}");
}

/// 3 — an effectful-namespace op is impure (the `is_effectful_op` seed).
#[test]
fn effectful_namespace_op_is_impure() {
    let v = pure("(:wat::io::IOReader/open-file \"x\")");
    assert!(is_false(&v), "io op → false; got {v:?}");
}

/// 4 — `Uuid/v4` is impure (RANDOM → non-deterministic, though it does no IO; OUTSIDE the effectful
/// namespaces). LOAD-BEARING: "pure" = deterministic fn of facts; this is the case default-allow gets wrong.
#[test]
fn non_deterministic_uuid_is_impure() {
    let v = pure("(:wat::core::Uuid/v4)");
    assert!(is_false(&v), "Uuid/v4 (random, non-deterministic) → false; got {v:?}");
}

/// 4b — `Uuid/v5` is PURE: SHA1 of namespace+name is deterministic. Guards the v4/v5 boundary so the
/// fence denies *randomness*, not the whole Uuid family.
#[test]
fn deterministic_uuid_v5_is_pure() {
    let v = pure("(:wat::core::Uuid/v5 (:wat::core::Uuid/nil) \"x\")");
    assert!(is_true(&v), "Uuid/v5 (deterministic) → true; got {v:?}");
}

/// 5 — a user fn with a pure body is pure (transitive, into the fn body).
#[test]
fn pure_user_fn_is_pure() {
    let v = pure("(:test::pure-double 5)");
    assert!(is_true(&v), "pure user fn → true; got {v:?}");
}

/// 6 — a user fn that transitively calls an impure op is impure. LOAD-BEARING: the transitive hole closed.
#[test]
fn transitively_impure_user_fn_is_impure() {
    let v = pure("(:test::impure-uuid)");
    assert!(is_false(&v), "user fn → Uuid/v4 transitively → false; got {v:?}");
}

/// 7 — an unknown head is impure (DEFAULT-DENY: unproven ⇒ rejected).
#[test]
fn unknown_head_is_impure() {
    let v = pure("(:not::a::real::op 1)");
    assert!(is_false(&v), "unknown head → false (default-deny); got {v:?}");
}

/// 8 — a self-recursive pure fn is pure (cycle handled; classification terminates).
#[test]
fn self_recursive_pure_fn_terminates_pure() {
    let v = pure("(:test::countdown 3)");
    assert!(is_true(&v), "self-recursive pure fn → true (cycle safe); got {v:?}");
}

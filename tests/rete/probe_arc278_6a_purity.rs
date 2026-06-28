//! Arc 278 — Stone 6a: the rete condition fence = TWO orthogonal classifiers, `:wat::rete::pure?` +
//! `:wat::rete::deterministic?`. A rete condition must be (pure AND deterministic); each axis is its
//! own predicate. Contract: DESIGN-STONE-6a-purity-inference.md.
//!
//! THE load-bearing reframe (the orthogonality proof): `:wat::core::Uuid/v4` does no IO and mutates
//! nothing → it is PURE; but it is random → NON-deterministic. So `pure?` → true AND `deterministic?`
//! → false on the same op. Each classifier is DEFAULT-DENY and transitive over user-fn bodies.
//!
//! Run: cargo test --release -p wat --test probe_arc278_6a_purity

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Run `(:wat::rete::<pred> (:wat::core::quote <expr>))` and return the bool.
fn ask(pred: &str, expr: &str) -> Value {
    let compute = format!("(:wat::rete::{pred} (:wat::core::quote {expr}))");
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(&compute).expect("parse compute");
    eval_in_frozen(&ast, &world, &Environment::new()).expect("predicate should run").value_owned()
}
fn pure(expr: &str) -> Value { ask("pure?", expr) }
fn det(expr: &str) -> Value { ask("deterministic?", expr) }
fn is_true(v: &Value) -> bool { matches!(v, Value::bool(true)) }
fn is_false(v: &Value) -> bool { matches!(v, Value::bool(false)) }

// ─── THE orthogonality proof: Uuid/v4 is pure ∧ non-deterministic ──────────────

/// pure? on `Uuid/v4` → TRUE — it does no IO / mutates nothing, so it is effect-free.
#[test]
fn uuid_v4_is_pure() {
    let v = pure("(:wat::core::Uuid/v4)");
    assert!(is_true(&v), "Uuid/v4 is effect-free → pure? true; got {v:?}");
}

/// deterministic? on `Uuid/v4` → FALSE — it is random. (The same op, the other axis.)
#[test]
fn uuid_v4_is_not_deterministic() {
    let v = det("(:wat::core::Uuid/v4)");
    assert!(is_false(&v), "Uuid/v4 is random → deterministic? false; got {v:?}");
}

/// deterministic? on `Uuid/v5` → TRUE — SHA1(ns,name) is referentially transparent (the v4/v5 boundary).
#[test]
fn uuid_v5_is_deterministic() {
    let v = det("(:wat::core::Uuid/v5 (:wat::core::Uuid/nil) \"x\")");
    assert!(is_true(&v), "Uuid/v5 is deterministic → true; got {v:?}");
}

// ─── pure? axis (effect-free) ───────────────────────────────────────────────────

#[test]
fn pure_arithmetic_is_pure() {
    assert!(is_true(&pure("(:wat::core::> (:wat::core::- 5 3) 1)")), "pure arithmetic");
}

#[test]
fn pure_string_predicate_is_pure() {
    assert!(is_true(&pure("(:wat::core::string::starts-with? \"abc\" \"a\")")), "string::starts-with?");
}

/// An effectful-namespace op is NOT pure (the is_effectful_op seed).
#[test]
fn io_op_is_not_pure() {
    assert!(is_false(&pure("(:wat::io::IOReader/open-file \"x\")")), "io op → pure? false");
}

/// A user fn that transitively calls an effectful op is NOT pure (transitive over the body).
#[test]
fn transitively_effectful_user_fn_is_not_pure() {
    assert!(is_false(&pure("(:test::io-fn)")), "user fn → io transitively → pure? false");
}

/// A pure user fn is pure (transitive into a clean body).
#[test]
fn pure_user_fn_is_pure() {
    assert!(is_true(&pure("(:test::pure-double 5)")), "pure user fn → pure? true");
}

/// An unknown head is NOT pure (DEFAULT-DENY).
#[test]
fn unknown_head_is_not_pure() {
    assert!(is_false(&pure("(:not::a::real::op 1)")), "unknown head → pure? false");
}

/// A self-recursive pure fn classifies pure and terminates (cycle handled).
#[test]
fn self_recursive_pure_fn_is_pure() {
    assert!(is_true(&pure("(:test::countdown 3)")), "self-recursive pure fn → pure? true");
}

/// `cond` is clause-aware: a pure cond is pure; an io body makes it impure.
#[test]
fn pure_cond_is_pure() {
    assert!(is_true(&pure("(:wat::core::cond ((:wat::core::> 5 3) 1) (true 0))")), "pure cond");
}
#[test]
fn cond_with_io_body_is_not_pure() {
    assert!(is_false(&pure("(:wat::core::cond ((:wat::core::> 5 3) (:wat::io::IOReader/open-file \"x\")) (true 0))")), "cond io body");
}

/// `match` is clause-aware: the constructor PATTERN `(:Some v)` is structural and must be SKIPPED,
/// not misclassified as an impure call; a pure match is pure.
#[test]
fn pure_match_with_constructor_pattern_is_pure() {
    assert!(is_true(&pure("(:wat::core::match ?x -> :wat::core::i64 ((:wat::core::Some v) v) (:wat::core::None 0))")), "pure match (pattern skipped)");
}
#[test]
fn match_with_io_body_is_not_pure() {
    assert!(is_false(&pure("(:wat::core::match ?x -> :wat::core::nil ((:wat::core::Some v) (:wat::io::IOReader/open-file \"x\")) (:wat::core::None nil))")), "match io body");
}

// ─── deterministic? axis (referential transparency) ─────────────────────────────

#[test]
fn pure_arithmetic_is_deterministic() {
    assert!(is_true(&det("(:wat::core::> (:wat::core::- 5 3) 1)")), "arithmetic → deterministic? true");
}

/// A user fn transitively calling `Uuid/v4` is NOT deterministic (transitive over the body).
#[test]
fn transitively_nondeterministic_user_fn_is_not_deterministic() {
    assert!(is_false(&det("(:test::nondet-uuid)")), "user fn → Uuid/v4 transitively → deterministic? false");
}

/// An effectful op is not deterministic either (not in the metadata map → default-deny).
#[test]
fn io_op_is_not_deterministic() {
    assert!(is_false(&det("(:wat::io::IOReader/open-file \"x\")")), "io op → deterministic? false");
}

/// `match` scrutinee is checked on the determinism axis too.
#[test]
fn match_on_nondeterministic_scrutinee_is_not_deterministic() {
    assert!(is_false(&det("(:wat::core::match (:wat::core::Uuid/v4) -> :wat::core::nil (:wat::core::None nil))")), "match on Uuid/v4 scrut → deterministic? false");
}

/// A self-recursive pure fn is also deterministic (cycle handled on this axis too).
#[test]
fn self_recursive_fn_is_deterministic() {
    assert!(is_true(&det("(:test::countdown 3)")), "self-recursive fn → deterministic? true");
}

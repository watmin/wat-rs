//! Arc 278 — Stone 6a: the rete condition fence = TWO orthogonal classifiers, `:wat::rete::pure?` +
//! `:wat::rete::deterministic?`. A rete condition must be (pure AND deterministic); each axis is its
//! own predicate. Contract: DESIGN-STONE-6a-purity-inference.md.
//!
//! THE load-bearing reframe (the orthogonality proof): `:wat::uuid::v4` does no IO and mutates
//! nothing → it is PURE; but it is random → NON-deterministic. So `pure?` → true AND `deterministic?`
//! → false on the same op. Each classifier is DEFAULT-DENY and transitive over user-fn bodies.
//!
//! Run: cargo test --release -p wat --test probe_arc278_6a_purity

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn is_true(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(true)) }
fn is_false(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(false)) }

// ─── THE orthogonality proof: Uuid/v4 is pure ∧ non-deterministic ──────────────

/// pure? on `Uuid/v4` → TRUE — it does no IO / mutates nothing, so it is effect-free.
#[test]
fn uuid_v4_is_pure() {
    assert!(is_true(":user::uuid-v4-pure?"), "Uuid/v4 is effect-free → pure? true");
}

/// deterministic? on `Uuid/v4` → FALSE — it is random. (The same op, the other axis.)
#[test]
fn uuid_v4_is_not_deterministic() {
    assert!(is_false(":user::uuid-v4-deterministic?"), "Uuid/v4 is random → deterministic? false");
}

/// deterministic? on `Uuid/v5` → TRUE — SHA1(ns,name) is referentially transparent (the v4/v5 boundary).
#[test]
fn uuid_v5_is_deterministic() {
    assert!(is_true(":user::uuid-v5-deterministic?"), "Uuid/v5 is deterministic → true");
}

// ─── pure? axis (effect-free) ───────────────────────────────────────────────────

#[test]
fn pure_arithmetic_is_pure() {
    assert!(is_true(":user::pure-arithmetic-pure?"), "pure arithmetic");
}

#[test]
fn pure_string_predicate_is_pure() {
    assert!(is_true(":user::pure-string-predicate-pure?"), "string::starts-with?");
}

/// An effectful-namespace op is NOT pure (the is_effectful_op seed).
#[test]
fn io_op_is_not_pure() {
    assert!(is_false(":user::io-op-pure?"), "io op → pure? false");
}

/// A user fn that transitively calls an effectful op is NOT pure (transitive over the body).
#[test]
fn transitively_effectful_user_fn_is_not_pure() {
    assert!(is_false(":user::transitively-effectful-user-fn-pure?"), "user fn → io transitively → pure? false");
}

/// A pure user fn is pure (transitive into a clean body).
#[test]
fn pure_user_fn_is_pure() {
    assert!(is_true(":user::pure-user-fn-pure?"), "pure user fn → pure? true");
}

/// An unknown head is NOT pure (DEFAULT-DENY).
#[test]
fn unknown_head_is_not_pure() {
    assert!(is_false(":user::unknown-head-pure?"), "unknown head → pure? false");
}

/// A self-recursive pure fn classifies pure and terminates (cycle handled).
#[test]
fn self_recursive_pure_fn_is_pure() {
    assert!(is_true(":user::self-recursive-pure-fn-pure?"), "self-recursive pure fn → pure? true");
}

/// `cond` is clause-aware: a pure cond is pure; an io body makes it impure.
#[test]
fn pure_cond_is_pure() {
    assert!(is_true(":user::pure-cond-pure?"), "pure cond");
}
#[test]
fn cond_with_io_body_is_not_pure() {
    assert!(is_false(":user::cond-with-io-body-pure?"), "cond io body");
}

/// `match` is clause-aware: the constructor PATTERN `(:Some v)` is structural and must be SKIPPED,
/// not misclassified as an impure call; a pure match is pure.
#[test]
fn pure_match_with_constructor_pattern_is_pure() {
    assert!(is_true(":user::pure-match-with-ctor-pattern-pure?"), "pure match (pattern skipped)");
}
#[test]
fn match_with_io_body_is_not_pure() {
    assert!(is_false(":user::match-with-io-body-pure?"), "match io body");
}

// ─── deterministic? axis (referential transparency) ─────────────────────────────

#[test]
fn pure_arithmetic_is_deterministic() {
    assert!(is_true(":user::pure-arithmetic-deterministic?"), "arithmetic → deterministic? true");
}

/// A user fn transitively calling `Uuid/v4` is NOT deterministic (transitive over the body).
#[test]
fn transitively_nondeterministic_user_fn_is_not_deterministic() {
    assert!(is_false(":user::transitively-nondeterministic-user-fn-deterministic?"), "user fn → Uuid/v4 transitively → deterministic? false");
}

/// An effectful op is not deterministic either (not in the metadata map → default-deny).
#[test]
fn io_op_is_not_deterministic() {
    assert!(is_false(":user::io-op-deterministic?"), "io op → deterministic? false");
}

/// `match` scrutinee is checked on the determinism axis too.
#[test]
fn match_on_nondeterministic_scrutinee_is_not_deterministic() {
    assert!(is_false(":user::match-on-nondeterministic-scrutinee-deterministic?"), "match on Uuid/v4 scrut → deterministic? false");
}

/// A self-recursive pure fn is also deterministic (cycle handled on this axis too).
#[test]
fn self_recursive_fn_is_deterministic() {
    assert!(is_true(":user::self-recursive-fn-deterministic?"), "self-recursive fn → deterministic? true");
}

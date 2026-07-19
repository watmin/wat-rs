//! Arc 278 — the 6a purity fence must read a generated field ACCESSOR's DECLARED purity, the
//! same declaration `constructor_meta` already reads for constructors. A Record accessor
//! (`Log/level`) is pure ∧ deterministic (`AggregateDef.nature.is_pure()`); today the accessor
//! head falls PAST `constructor_meta` (its `rsplit_once("::")` yields a non-type) into a `Native`
//! stub → `classify_fn` → default-deny. That rejects EVERY realistic sift predicate (they all read
//! a field). RED at HEAD: `pure?`/`deterministic?` of a `Log`-accessor predicate → FALSE.
//!
//! GREEN when `head_ok` gains an `accessor_meta` sibling that resolves `Type/field` → the type's
//! declared purity (Record accessor → pure via `nature.is_pure()`; enum accessor → its
//! `:wat::enum::*` marker; Struct → impure), read from the frozen TypeEnv exactly as
//! `constructor_meta` does. This is the declaration model (arc 255-aligned), NOT a hand-list.
//! Guard: an effectful body must STILL be rejected — the accessor fix must not blanket-allow.
//!
//! Run: cargo test --release -p wat accessor_purity

use wat::freeze::call_beside;
use wat::runtime::Value;

/// Invoke a co-located zero-arg entry (each quotes its predicate under test and hands it to the
/// fence predicate) and return its bool result.
fn classify(fn_name: &str) -> bool {
    match call_beside(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

/// A Record field accessor (`Log/level`) is pure by construction — the fence must accept it.
#[test]
fn record_accessor_is_pure() {
    assert!(
        classify(":user::log-accessor-is-pure"),
        "a Record field accessor (:wat::telemetry::Log/level) is pure by construction"
    );
}

/// The same accessor is deterministic — a field read is referentially transparent.
#[test]
fn record_accessor_is_deterministic() {
    assert!(
        classify(":user::log-accessor-is-deterministic"),
        "a Record field accessor is deterministic"
    );
}

/// GUARD: an effectful body (a `println` on the accessed field) must STILL be rejected — the
/// accessor fix must not blanket-allow; the effectful op's impurity must propagate.
#[test]
fn impure_accessor_body_is_not_pure() {
    assert!(
        !classify(":user::impure-accessor-body-is-not-pure"),
        "an effectful body must STILL be rejected — the accessor fix is not a blanket-allow"
    );
}

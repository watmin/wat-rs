//! Arc 278 sift-arena, Part A — the RED gate for the `:wat::edn::` purity-fence fix.
//! `intrinsic_meta` (src/rete/purity.rs) had no `:wat::edn::` entry, so any foreign-reader
//! predicate (`read-foreign` + `ForeignRecord/get`/`class`) default-denied — rejecting every
//! realistic cross-universe sift predicate. FIX: the whole `:wat::edn::` namespace is pure data
//! transforms (parse/serialize/navigate, no IO, no entropy) — classified pure ∧ deterministic by
//! prefix, beside the existing `:wat::core::string::`/`regex::` namespace rule.
//! GUARD: an effectful body (println) must STILL be rejected — conditional purity, not
//! blanket-allow.
//!
//! Run: cargo test --release -p wat foreign_pred_purity

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn classify(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {other:?}"),
    }
}

/// A foreign-reader predicate (`read-foreign` + `ForeignRecord/get`) is pure — the `:wat::edn::`
/// namespace is data transforms only, no IO.
#[test]
fn foreign_pred_is_pure() {
    assert!(
        classify(":user::foreign-pred-is-pure"),
        "a read-foreign + ForeignRecord/get predicate is pure by namespace"
    );
}

/// The same predicate is deterministic — parse/navigate is referentially transparent.
#[test]
fn foreign_pred_is_deterministic() {
    assert!(
        classify(":user::foreign-pred-is-deterministic"),
        "a read-foreign + ForeignRecord/get predicate is deterministic"
    );
}

/// The same predicate is total — `read-foreign` returns Outcome, `get` returns Option.
#[test]
fn foreign_pred_is_total() {
    assert!(
        classify(":user::foreign-pred-is-total"),
        "a read-foreign + ForeignRecord/get predicate is total"
    );
}

/// GUARD: an effectful body (println on the decoded field) must STILL be rejected — the
/// `:wat::edn::` fix is not a blanket-allow; the impure op's impurity must still propagate.
#[test]
fn impure_foreign_pred_is_not_pure() {
    assert!(
        !classify(":user::impure-foreign-pred-is-not-pure"),
        "an effectful body must STILL be rejected — the edn namespace fix is not a blanket-allow"
    );
}

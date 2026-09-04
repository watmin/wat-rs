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

/// arc 255 Stone 1c-g — NEGATIVE WITNESS, inverted from the original positive claim. The
/// predicate under test ends in `(:wat::core::= s "high")`, where `s` comes out of
/// `ForeignRecord/get` typed `:wat::core::Value` (the EDN reader has no `Value`→`String`
/// coercion). Before this stone, `=`/`not=` were UNregistered, so `total?`'s registry-first
/// consult returned `None` and fell through to `intrinsic_meta`'s by-name `matches!` placeholder
/// — which named `":wat::core::="`/`":wat::core::not="` and answered `true` for both, a
/// hardcoded lie about a verb with a reachable raise (the very placeholder this stone deletes).
/// That lie is why the ORIGINAL `foreign_pred_is_total` assertion passed. Now that both are
/// registered `@Totality Partial`, the SAME registry-first consult answers `Some(Partial) =>
/// false` directly, before ever reaching the (now-empty) fallback — `Value`'s declared domain
/// admits `Fn`, and `values_equal` has no `Fn` arm, so a well-typed call over a `Value` can still
/// reach the raise. The assertion flips to false, and the false is the honest answer. `pure?`/
/// `deterministic?` above are unaffected — this axis alone flips.
#[test]
fn foreign_pred_is_total() {
    assert!(
        !classify(":user::foreign-pred-is-total"),
        "arc 255 Stone 1c-g: a foreign-reader predicate ending in `(= s \"high\")` over a \
         `:wat::core::Value` is NOT total — `=` is registered `@Totality Partial` (Value's \
         declared domain admits Fn, values_equal has no Fn arm), so `total?` must answer false"
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

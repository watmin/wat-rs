//! Arc 278 #55 (S3b+S4) slice one — THE ONE TABLE (`src/rete/vocabulary.rs`): four
//! rete-namespaced ops (one of each mechanism class) plus the module-set admission test.
//! Contract: `DESIGN-STONE-slice-one-rete-vocabulary.md`. Covers EXPECTATIONS rows 3-7 and 9.
//!
//! Rows 3-5 TOGETHER, not separately: an admission test that only refuses is the vacuous-gate
//! class this arc has hit three times (R59; `91bbb8cd`'s 11 gates; R62's empty rejection
//! column) — this file proves an ADMIT alongside the two REFUSE cases.
//!
//! Row 6 (composition) is proven BY A RUN, not assumed: a user `defn` built from all four ops is
//! asked of `:wat::rete::pure?`/`deterministic?`, which recurse into the fn body via
//! `classify_fn` — the property the whole design rests on ("users must be able to compose any
//! amount of complexity from these").
//!
//! Run: cargo test --release --test rete

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn is_true(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(true)) }
fn is_false(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(false)) }
fn eval_i64(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64, got {other:?}"),
    }
}

// ─── THE ADMISSION TEST (rows 3-5) ───────────────────────────────────────────────

/// Row 3: a rete-module head (`:wat::rete::i64::>`, inside the declared `i64::` sub-namespace)
/// IS admitted.
#[test]
fn admission_admits_a_rete_module_head() {
    assert!(is_true(":user::admit-rete-module?"), "`:wat::rete::i64::>` falls under the `i64::` vocabulary sub-namespace");
}

/// Row 4: the bare rete ENGINE API is refused — `:wat::rete::` alone is already
/// `fire-rules`/`insert`/`compile`/`Session`/… (STOP-1: a bare-prefix test would wrongly admit it).
#[test]
fn admission_refuses_the_bare_engine_api() {
    assert!(is_false(":user::refuse-engine-api?"), "`:wat::rete::fire-rules` is engine API, not a vocabulary op — it names no declared sub-namespace");
}

/// Row 5: a `:wat::core::` head is refused — it never falls under `:wat::rete::` at all.
#[test]
fn admission_refuses_a_core_head() {
    assert!(is_false(":user::refuse-core-head?"), "`:wat::core::i64::+` is not rete-namespaced");
}

// ─── the four ops dispatch (row 7) ────────────────────────────────────────────────

/// The `Alias` class: `:wat::rete::i64::>` reaches the SAME `eval_compare` routine
/// `:wat::core::i64::>` uses.
#[test]
fn alias_dispatches_to_the_same_routine() {
    assert!(is_true(":user::alias-gt"), "5 > 3");
}

/// The `Fallback` class on the non-overflowing path: ordinary addition, no substitution.
#[test]
fn fallback_returns_the_arithmetic_result_when_it_does_not_overflow() {
    assert_eq!(eval_i64(":user::fallback-no-overflow"), 5);
}

/// The `Form` class: `:wat::rete::core::and` reaches the SAME `eval_and` short-circuit routine
/// `:wat::core::and` uses.
#[test]
fn form_dispatches_to_the_same_short_circuit_routine() {
    assert!(is_true(":user::form-and"));
}

/// Row 9: the fallback FIRES on overflow — `i64::MAX + 1` never raises; `-1` is substituted.
#[test]
fn fallback_fires_on_overflow_instead_of_raising() {
    assert_eq!(eval_i64(":user::fallback-overflow"), -1, "i64::MAX + 1 overflows; the `:undefined` fallback substitutes -1, no raise");
}

// ─── row 6: COMPOSITION survives, proven BY A RUN ────────────────────────────────

/// A user `defn` composed of all four ops classifies PURE transitively — `classify_fn` still
/// recurses into a body that calls rete-namespaced heads.
#[test]
fn composed_user_fn_over_the_four_ops_is_pure() {
    assert!(is_true(":user::combo-is-pure?"), "a user defn built from all four ops classifies pure transitively");
}

/// ...and DETERMINISTIC transitively (the other axis, same walk).
#[test]
fn composed_user_fn_over_the_four_ops_is_deterministic() {
    assert!(is_true(":user::combo-is-deterministic?"), "...and deterministic transitively");
}

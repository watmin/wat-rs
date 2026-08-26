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
use wat::runtime::{RuntimeErrorKind, Value};

fn is_true(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(true)) }
fn is_false(entry: &str) -> bool { matches!(call_beside_value(file!(), entry).expect("eval"), Value::bool(false)) }
fn eval_i64(entry: &str) -> i64 {
    match call_beside_value(file!(), entry).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64, got {other:?}"),
    }
}

// ─── THE ADMISSION TEST (rows 3-5) ───────────────────────────────────────────────

/// Row 3: a rete-module head (`:wat::rete::i64::>`, inside the declared `core::` sub-namespace)
/// IS admitted.
#[test]
fn admission_admits_a_rete_module_head() {
    assert!(is_true(":user::admit-rete-module?"), "`:wat::rete::i64::>` falls under the `core::` vocabulary sub-namespace");
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

// ─── #56: the two head-table form mirrors ────────────────────────────────────────

/// `:wat::rete::core::not` is an `Alias`, not a `Form` — a plain strict fn with an ordinary
/// `TypeScheme`, dispatched to the same `eval_not` its core name uses. (The parent stone grouped
/// `and`/`or`/`not` together as "plain"; two of the three are wrong, and this is the one that
/// isn't.)
#[test]
fn not_dispatches_as_a_plain_alias() {
    assert!(is_true(":user::alias-not"), "negating false must yield true");
}

/// `:wat::rete::core::or` SHORT-CIRCUITS. This is the gate that decides `Form` vs `Alias`: an
/// `Alias` evaluates both operands strictly, so a strict `or` would raise on the divide-by-zero
/// second operand instead of returning `true`. Answering `true` is not the assertion — *never
/// reaching the second operand* is.
#[test]
fn or_short_circuits_and_does_not_evaluate_the_raising_operand() {
    assert!(
        is_true(":user::form-or-short-circuits"),
        "a true first operand must return true WITHOUT evaluating the raising second operand — if this goes red, or became strict"
    );
}

/// The NON-VACUITY CONTROL for the test above. The identical raising operand, actually reached,
/// DOES abort — so the short-circuit test is proving laziness rather than passing on an operand
/// that happened to be harmless. Without this control, both tests could be green with `or`
/// strict and the divide silently total.
///
/// Asserts the SPECIFIC failure, not merely `is_err()`: a bare `is_err()` would also be satisfied
/// by a typo'd entry name, which is the same vacuity this control exists to prevent.
#[test]
fn or_control_the_same_operand_reached_does_raise() {
    let err = call_beside_value(file!(), ":user::form-or-control-raises")
        .expect_err("a false first operand MUST reach the second and raise — otherwise the short-circuit test above is vacuous");
    // Matched on the typed KIND, not on a rendered substring: the failure must be the DIVIDE, and
    // a substring check would also be satisfied by a missing entry or an unrelated error — which
    // is the very vacuity this control exists to rule out.
    assert!(
        matches!(err.kind(), RuntimeErrorKind::DivisionByZero),
        "the control must fail on the divide itself; got kind: {:?}",
        err.kind()
    );
}

// ─── #56 phase 1: the head-table pair (`if`/`let`) ────────────────────────────────

/// Row 3: `:wat::rete::core::if` routes to `infer_if`, NOT `infer_boolean_shortcircuit`. Non-bool
/// (i64) branches must unify and type-check clean — pre-phase-1 this fixture would have FAILED TO
/// LOAD (a type error demanding `:bool` branches), so a successful `eval` here is itself the
/// discriminator, not merely the returned value.
#[test]
fn rete_if_routes_to_if_inference_not_boolean_shortcircuit() {
    assert_eq!(eval_i64(":user::rete-if-non-bool-branches"), 1, "the taken (true) branch's value");
}

/// Row 4: `:wat::rete::core::if` does not evaluate the untaken branch — the untaken else branch
/// raises, so reaching a return value at all proves it was never evaluated.
#[test]
fn rete_if_does_not_evaluate_the_untaken_branch() {
    assert_eq!(eval_i64(":user::rete-if-short-circuits"), 1, "the taken (true) branch's value, only reachable if the raising else branch was never evaluated");
}

/// Row 5: the NON-VACUITY CONTROL for row 4 — the identical raising operand, actually reached
/// (condition now false), DOES raise. Matched on the typed KIND, not a rendered substring
/// (`no_loose_string_assert`): a substring check would also pass on a missing entry or an
/// unrelated failure, which is the very vacuity this control exists to rule out.
#[test]
fn rete_if_control_the_untaken_branch_reached_does_raise() {
    let err = call_beside_value(file!(), ":user::rete-if-control-raises")
        .expect_err("the false condition MUST take the else branch and raise — otherwise row 4 is vacuous");
    assert!(
        matches!(err.kind(), RuntimeErrorKind::DivisionByZero),
        "the control must fail on the divide itself; got kind: {:?}",
        err.kind()
    );
}

/// Row 6: `:wat::rete::core::let` actually scopes a binding — bind, then read it back.
#[test]
fn rete_let_actually_scopes_a_binding() {
    assert_eq!(eval_i64(":user::rete-let-scopes"), 42);
}

/// Rows 7+8 — THE TCO GATE (the pair that matters; neither counts alone per EXPECTATIONS). A
/// tail-recursive fn whose tail form is a rete `if` must survive depth 200000 exactly as its core
/// twin does (`probe-s5-tail-position-is-load-bearing.wat`'s own proof of the same depth) —
/// without `eval_tail`'s gate this SIGSEGVs (exit 139) well before returning, which would abort
/// this whole test binary rather than fail one assertion. Row 8 (removing the gate, watching this
/// go red, restoring it) is a manual one-time verification reported in prose, not encoded as a
/// toggle here — there is no safe way to "expect" a SIGSEGV from inside the process it would kill.
#[test]
fn rete_if_tail_position_preserves_tco_at_depth() {
    assert_eq!(eval_i64(":user::rete-if-tail-tco-survives-depth"), 0, "the base case's value, reachable only if 200000 tail calls all reused the same native stack frame");
}

// ─── #56 phase 2: `match`, the first of the structural-guard pair ─────────────────

/// Row 10: a rete `match` whose arm PATTERN would fail as an expression classifies clean —
/// `:wat::rete::pure?` must recurse into the fn body via the structural (skip-the-pattern)
/// treatment, not the generic call-shape walk that would choke on `(:wat::core::Some n)`'s
/// list-shaped pattern head.
#[test]
fn rete_match_pattern_is_not_classified_as_an_expression_pure() {
    assert!(is_true(":user::rete-match-pattern-not-classified-as-expr-pure"), "the pattern must be skipped structurally, not walked as a call");
}

/// ...and the same discipline holds on the deterministic axis (the other independent classifier
/// sharing the same `classify_expr` walk).
#[test]
fn rete_match_pattern_is_not_classified_as_an_expression_deterministic() {
    assert!(is_true(":user::rete-match-pattern-not-classified-as-expr-det"));
}

// ─── S5's last form (closing #56's leftover): `fn` ────────────────────────────────

/// Row 3: the builder's target form — a rete `fn` type-checks (`infer_rete_form` routes
/// `":wat::core::fn"` to `infer_fn`) AND evaluates as a value, applied to a real argument.
#[test]
fn rete_fn_target_form_type_checks_and_evaluates() {
    assert_eq!(eval_i64(":user::rete-fn-target-form"), 5, "0 + 5, the fallback never fires (no overflow)");
}

/// Row 4: a rete `fn`'s BODY is fence-checked — an impure body classifies NOT pure.
#[test]
fn rete_fn_impure_body_is_not_pure() {
    assert!(is_false(":user::rete-fn-impure-body-is-not-pure"), "an IOReader/open-file body must deny purity");
}

/// ...and NOT deterministic (the other axis, same walk).
#[test]
fn rete_fn_impure_body_is_not_deterministic() {
    assert!(is_false(":user::rete-fn-impure-body-is-not-deterministic"));
}

/// Row 5: the CONTROL for row 4 — the identical shape, pure body, classifies pure. Rows 4+5
/// together, not separately: a body-check test that only shows the impure case proves nothing
/// about the pure one.
#[test]
fn rete_fn_pure_body_is_pure() {
    assert!(is_true(":user::rete-fn-pure-body-is-pure"), "a pure-arithmetic body must classify pure");
}

/// ...and deterministic (the other axis, same walk).
#[test]
fn rete_fn_pure_body_is_deterministic() {
    assert!(is_true(":user::rete-fn-pure-body-is-deterministic"));
}

/// Row 6: the structural guard fires — a rete `fn` whose RETURN-TYPE SLOT holds an impure-LOOKING
/// head (never evaluated; a parametric type-form List in that position, per `parse_type_node`)
/// classifies clean. `:wat::rete::pure?` must recurse into the fn BODY via the structural
/// (skip-params-and-return-type) treatment, not the generic call-shape walk that would treat the
/// return-type slot as one more argument expression and deny on its `is_effectful_op` head.
#[test]
fn rete_fn_return_type_slot_is_not_classified_as_an_expression_pure() {
    assert!(is_true(":user::rete-fn-return-type-slot-not-classified-as-expr-pure"), "the return-type slot must be skipped structurally, not walked as an expression");
}

/// ...and the same discipline holds on the deterministic axis.
#[test]
fn rete_fn_return_type_slot_is_not_classified_as_an_expression_deterministic() {
    assert!(is_true(":user::rete-fn-return-type-slot-not-classified-as-expr-det"));
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

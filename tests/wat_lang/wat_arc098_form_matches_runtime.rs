//! Arc 098 slice 2 — `:wat::form::matches?` runtime walker.
//!
//! End-to-end coverage: a wat program declares a struct, constructs a
//! value, calls `(matches? subject pattern)`, and the test asserts
//! the boolean result. Every case from the DESIGN's runtime
//! semantics is exercised:
//!
//! - The worked example (PaperResolved + Grace + > 5.0).
//! - All clause kinds: bindings, comparisons (= < > <= >= not=),
//!   logical combinators (and / or / not), where-escape.
//! - Negative paths: struct-type mismatch, Option-None subject,
//!   non-Struct subject — all return `false` per Clara semantics.
//!
//! Slice 1 covers the type-check side; this slice covers runtime.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

fn assert_bool(v: Value, expected: bool, ctx: &str) {
    match v {
        Value::bool(b) if b == expected => {}
        other => panic!("{}: expected bool {}; got {:?}", ctx, expected, other),
    }
}

// ─── Worked example: PaperResolved Grace > 5.0 ──────────────────────

#[test]
fn worked_example_matches() {
    assert_bool(run_expr("(:t::test1-worked)"), true, "Grace 7.5 should match");
}

#[test]
fn worked_example_rejects_low_residue() {
    assert_bool(
        run_expr("(:t::test2-low-residue)"),
        false,
        "Grace 3.0 should not match (residue too low)",
    );
}

#[test]
fn worked_example_rejects_wrong_outcome() {
    assert_bool(
        run_expr("(:t::test3-wrong-outcome)"),
        false,
        "Loss should not match Grace pattern",
    );
}

// ─── Comparison vocabulary: = < > <= >= not= ────────────────────────

#[test]
fn comparison_lt_gt_le_ge() {
    // < 5.0
    assert_bool(run_expr("(:t::test4-lt-high)"), false, "7.5 < 5.0 = F");
    assert_bool(run_expr("(:t::test4-lt-low)"), true, "3.0 < 5.0 = T");
    // > 5.0
    assert_bool(run_expr("(:t::test4-gt-high)"), true, "7.5 > 5.0 = T");
    assert_bool(run_expr("(:t::test4-gt-low)"), false, "3.0 > 5.0 = F");
    // <= 5.0
    assert_bool(run_expr("(:t::test4-le-high)"), false, "7.5 <= 5.0 = F");
    assert_bool(run_expr("(:t::test4-le-low)"), true, "3.0 <= 5.0 = T");
    // >= 5.0
    assert_bool(run_expr("(:t::test4-ge-high)"), true, "7.5 >= 5.0 = T");
    assert_bool(run_expr("(:t::test4-ge-low)"), false, "3.0 >= 5.0 = F");
}

#[test]
fn not_eq_works() {
    assert_bool(run_expr("(:t::test5-not-eq)"), true, "Loss != Grace should match");
}

// ─── Logical combinators: and / or / not ────────────────────────────

#[test]
fn and_both_must_hold() {
    assert_bool(run_expr("(:t::test6-and-pass)"), true, "Grace 7.0 and-pass");
    assert_bool(run_expr("(:t::test6-and-fail-residue)"), false, "Grace 3.0 fails residue");
    assert_bool(run_expr("(:t::test6-and-fail-outcome)"), false, "Loss fails outcome");
}

#[test]
fn or_at_least_one_must_hold() {
    assert_bool(run_expr("(:t::test7-or-low)"), true, "low triggers second branch");
    assert_bool(run_expr("(:t::test7-or-high)"), true, "high triggers first branch");
    assert_bool(run_expr("(:t::test7-or-mid)"), false, "middle triggers neither");
}

#[test]
fn not_inverts() {
    assert_bool(run_expr("(:t::test8-not-grace)"), true, "Grace passes not-Loss");
    assert_bool(run_expr("(:t::test8-not-loss)"), false, "Loss fails not-Loss");
}

// ─── where-escape ───────────────────────────────────────────────────

#[test]
fn where_uses_arbitrary_wat_expression() {
    assert_bool(
        run_expr("(:t::test9-where-pass)"),
        true,
        "where passes when string contains Grace",
    );
}

#[test]
fn where_can_fail() {
    assert_bool(
        run_expr("(:t::test10-where-fail)"),
        false,
        "where fails when no substring match",
    );
}

// ─── Negative paths: false (no error) ───────────────────────────────

#[test]
fn struct_type_mismatch_returns_false() {
    assert_bool(run_expr("(:t::test11-struct-mismatch)"), false, "wrong struct type returns false");
}

#[test]
fn option_none_subject_returns_false() {
    assert_bool(run_expr("(:t::test12-option-none)"), false, "Option None returns false");
}

#[test]
fn option_some_subject_unwraps_one_level() {
    assert_bool(
        run_expr("(:t::test13-option-some)"),
        true,
        "Option Some matches inner struct",
    );
}

#[test]
fn non_struct_subject_returns_false() {
    assert_bool(run_expr("(:t::test14-non-struct)"), false, "i64 subject returns false");
}

// ─── Bindings flow forward across clauses ────────────────────────────

#[test]
fn binding_visible_in_later_clauses_including_where() {
    assert_bool(
        run_expr("(:t::test15-binding-where)"),
        true,
        "binding ?gr visible in where",
    );
}

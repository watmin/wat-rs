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

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Value {
    let world =
        startup_from_source(src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    invoke_user_main(&world, Vec::new()).expect("main")
}

fn assert_bool(v: Value, expected: bool, ctx: &str) {
    match v {
        Value::bool(b) if b == expected => {}
        other => panic!("{}: expected bool {}; got {:?}", ctx, expected, other),
    }
}

const PROLOGUE: &str = r#"
(:wat::core::struct :test::PaperResolved
  (outcome       :String)
  (grace-residue :f64))
"#;

fn program(body: &str) -> String {
    format!(
        "{prologue}\n(:wat::core::define (:user::main -> :bool) {body})",
        prologue = PROLOGUE,
        body = body
    )
}

// ─── Worked example: PaperResolved Grace > 5.0 ──────────────────────

#[test]
fn worked_example_matches() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Grace" 7.5)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?outcome :outcome)
              (= ?grace-residue :grace-residue)
              (= ?outcome "Grace")
              (> ?grace-residue 5.0))))
        "#,
    );
    assert_bool(run(&src), true, "Grace 7.5 should match");
}

#[test]
fn worked_example_rejects_low_residue() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Grace" 3.0)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?outcome :outcome)
              (= ?grace-residue :grace-residue)
              (= ?outcome "Grace")
              (> ?grace-residue 5.0))))
        "#,
    );
    assert_bool(run(&src), false, "Grace 3.0 should not match (residue too low)");
}

#[test]
fn worked_example_rejects_wrong_outcome() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Loss" 7.5)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?outcome :outcome)
              (= ?grace-residue :grace-residue)
              (= ?outcome "Grace")
              (> ?grace-residue 5.0))))
        "#,
    );
    assert_bool(run(&src), false, "Loss should not match Grace pattern");
}

// ─── Comparison vocabulary: = < > <= >= not= ────────────────────────

#[test]
fn comparison_lt_gt_le_ge() {
    // Each comparison op exercised against a single value.
    // Subject high = 7.5, low = 3.0. Each (op, threshold) row picks
    // a threshold where the two subjects fall on opposite sides of
    // the comparison so we test BOTH outcomes per op.
    for (op, threshold, expected_high, expected_low) in &[
        ("<", "5.0", false, true),    // 7.5 < 5.0 = F; 3.0 < 5.0 = T
        (">", "5.0", true, false),    // 7.5 > 5.0 = T; 3.0 > 5.0 = F
        ("<=", "5.0", false, true),   // 7.5 <= 5.0 = F; 3.0 <= 5.0 = T
        (">=", "5.0", true, false),   // 7.5 >= 5.0 = T; 3.0 >= 5.0 = F
    ] {
        let high_src = program(&format!(
            r#"
            (:wat::core::let*
              (((p :test::PaperResolved)
                (:test::PaperResolved/new "Grace" 7.5)))
              (:wat::form::matches? p
                (:test::PaperResolved
                  (= ?gr :grace-residue)
                  ({op} ?gr {threshold}))))
            "#,
        ));
        assert_bool(run(&high_src), *expected_high, &format!("op {} threshold {}", op, threshold));

        let low_src = program(&format!(
            r#"
            (:wat::core::let*
              (((p :test::PaperResolved)
                (:test::PaperResolved/new "Grace" 3.0)))
              (:wat::form::matches? p
                (:test::PaperResolved
                  (= ?gr :grace-residue)
                  ({op} ?gr {threshold}))))
            "#,
        ));
        assert_bool(run(&low_src), *expected_low, &format!("op {} threshold {} low", op, threshold));
    }
}

#[test]
fn not_eq_works() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Loss" 1.0)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?o :outcome)
              (:not= ?o "Grace"))))
        "#,
    );
    assert_bool(run(&src), true, "Loss != Grace should match");
}

// ─── Logical combinators: and / or / not ────────────────────────────

#[test]
fn and_both_must_hold() {
    let mk = |outcome: &str, residue: &str| {
        program(&format!(
            r#"
            (:wat::core::let*
              (((p :test::PaperResolved)
                (:test::PaperResolved/new "{outcome}" {residue})))
              (:wat::form::matches? p
                (:test::PaperResolved
                  (= ?o :outcome)
                  (= ?gr :grace-residue)
                  (:and (= ?o "Grace") (> ?gr 5.0)))))
            "#,
        ))
    };
    assert_bool(run(&mk("Grace", "7.0")), true, "Grace 7.0 and-pass");
    assert_bool(run(&mk("Grace", "3.0")), false, "Grace 3.0 fails residue");
    assert_bool(run(&mk("Loss", "7.0")), false, "Loss fails outcome");
}

#[test]
fn or_at_least_one_must_hold() {
    let mk = |residue: &str| {
        program(&format!(
            r#"
            (:wat::core::let*
              (((p :test::PaperResolved)
                (:test::PaperResolved/new "Grace" {residue})))
              (:wat::form::matches? p
                (:test::PaperResolved
                  (= ?gr :grace-residue)
                  (:or (> ?gr 100.0) (< ?gr 5.0)))))
            "#,
        ))
    };
    assert_bool(run(&mk("3.0")), true, "low triggers second branch");
    assert_bool(run(&mk("150.0")), true, "high triggers first branch");
    assert_bool(run(&mk("50.0")), false, "middle triggers neither");
}

#[test]
fn not_inverts() {
    let mk = |outcome: &str| {
        program(&format!(
            r#"
            (:wat::core::let*
              (((p :test::PaperResolved)
                (:test::PaperResolved/new "{outcome}" 5.0)))
              (:wat::form::matches? p
                (:test::PaperResolved
                  (= ?o :outcome)
                  (:not (= ?o "Loss")))))
            "#,
        ))
    };
    assert_bool(run(&mk("Grace")), true, "Grace passes not-Loss");
    assert_bool(run(&mk("Loss")), false, "Loss fails not-Loss");
}

// ─── where-escape ───────────────────────────────────────────────────

#[test]
fn where_uses_arbitrary_wat_expression() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Graceful" 7.5)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?o :outcome)
              (:where (:wat::core::string::contains? ?o "Grace")))))
        "#,
    );
    assert_bool(run(&src), true, "where passes when string contains Grace");
}

#[test]
fn where_can_fail() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Loss" 7.5)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?o :outcome)
              (:where (:wat::core::string::contains? ?o "Grace")))))
        "#,
    );
    assert_bool(run(&src), false, "where fails when no substring match");
}

// ─── Negative paths: false (no error) ───────────────────────────────

#[test]
fn struct_type_mismatch_returns_false() {
    // Subject is a different struct type — pattern walker returns
    // false without surfacing an error (Clara semantics).
    let src = format!(
        "{prologue}\n
        (:wat::core::struct :test::Other (x :i64))
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((o :test::Other) (:test::Other/new 42)))
            (:wat::form::matches? o
              (:test::PaperResolved
                (= ?gr :grace-residue)
                (> ?gr 5.0)))))
        ",
        prologue = PROLOGUE
    );
    assert_bool(run(&src), false, "wrong struct type returns false");
}

#[test]
fn option_none_subject_returns_false() {
    let src = program(
        r#"
        (:wat::core::let*
          (((maybe :wat::core::Option<test::PaperResolved>) :wat::core::None))
          (:wat::form::matches? maybe
            (:test::PaperResolved
              (= ?gr :grace-residue)
              (> ?gr 5.0))))
        "#,
    );
    assert_bool(run(&src), false, "Option None returns false");
}

#[test]
fn option_some_subject_unwraps_one_level() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Grace" 7.5))
           ((maybe :wat::core::Option<test::PaperResolved>) (:wat::core::Some p)))
          (:wat::form::matches? maybe
            (:test::PaperResolved
              (= ?gr :grace-residue)
              (> ?gr 5.0))))
        "#,
    );
    assert_bool(run(&src), true, "Option Some matches inner struct");
}

#[test]
fn non_struct_subject_returns_false() {
    let src = program(
        r#"
        (:wat::form::matches? 42
          (:test::PaperResolved
            (= ?gr :grace-residue)
            (> ?gr 5.0)))
        "#,
    );
    assert_bool(run(&src), false, "i64 subject returns false");
}

// ─── Bindings flow forward across clauses ────────────────────────────

#[test]
fn binding_visible_in_later_clauses_including_where() {
    let src = program(
        r#"
        (:wat::core::let*
          (((p :test::PaperResolved)
            (:test::PaperResolved/new "Grace" 12.5)))
          (:wat::form::matches? p
            (:test::PaperResolved
              (= ?o :outcome)
              (= ?gr :grace-residue)
              (= ?o "Grace")
              (:where (:wat::core::f64::> ?gr 10.0)))))
        "#,
    );
    assert_bool(run(&src), true, "binding ?gr visible in where");
}

//! Arc 071 regression — parametric built-in enum constructors must
//! type-check at use sites.
//!
//! Pre-arc-071, `register_enum_methods` synthesized the constructor's
//! return type as a bare `:wat::eval::WalkStep` regardless of whether
//! the enum had type parameters. The lab harness's `wat::test! {}`
//! path goes through `startup_from_source` (this test does too), and
//! `check_program` is invoked there — pre-fix, the checker saw the
//! body produce `:WalkStep` and rejected against a `:WalkStep<wat::core::i64>`
//! signature.
//!
//! The substrate's runtime-only `run` test helper (in `runtime.rs::
//! mod tests`) bypasses the type checker, so arc 070's walk_w1-w4
//! tests passed without exercising this. Lab consumers caught it.
//!
//! This test goes through the full freeze pipeline so the type
//! checker IS exercised. New parametric built-in enums must add a
//! similar probe — that's the discipline arc 071 introduces to
//! eliminate the harness-vs-substrate parity failure mode.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main/stdout-capture to
//! eval_in_frozen with :my::compute returning values.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

/// `:wat::eval::WalkStep<A>` (the first parametric built-in enum).
/// A function whose body returns `(:wat::eval::WalkStep::Continue
/// <i64>)` must satisfy a `-> :wat::eval::WalkStep<wat::core::i64>` signature.
/// Pre-arc-071 this failed type-check because the synthesized
/// constructor's return type was bare `:wat::eval::WalkStep`.
/// Arc 170 slice 1f-ζ: :my::compute calls :my::test::wrap and returns i64.
#[test]
fn walkstep_continue_parametric_inference_at_use_site() {
    let src = r#"
        (:wat::core::define
          (:my::test::wrap (n :wat::core::i64) -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Continue n))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [wrapped (:my::test::wrap 7)]
            7))
    "#;
    assert!(matches!(run(src), Value::i64(7)), "expected i64(7)");
}

#[test]
fn walkstep_skip_parametric_inference_at_use_site() {
    // `Skip` takes (terminal :HolonAST, acc :A). Same parametric
    // inference path but with a different field count.
    // Arc 170 slice 1f-ζ: :my::compute calls :my::test::halt and returns i64.
    let src = r#"
        (:wat::core::define
          (:my::test::halt
            (n :wat::core::i64)
            -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Skip
            (:wat::holon::leaf 999)
            n))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [halted (:my::test::halt 3)]
            3))
    "#;
    assert!(matches!(run(src), Value::i64(3)), "expected i64(3)");
}

/// The full walker pattern from arc 070's USER-GUIDE example,
/// frozen + type-checked. Equivalent to the lab harness probe at
/// `holon-lab-trading/wat-tests-integ/experiment/099-walkstep-probe`
/// — pre-arc-071, both this test and that probe failed; post-fix,
/// both pass.
/// Arc 170 slice 1f-ζ: :my::compute runs the walk and returns the count.
#[test]
fn walk_visitor_signature_matches_at_use_site() {
    let src = r#"
        (:wat::core::define
          (:my::test::count-visit
            (acc :wat::core::i64)
            (form :wat::WatAST)
            (step :wat::eval::StepResult)
            -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Continue (:wat::core::i64::+'2 acc 1)))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match
            (:wat::eval::walk
              (:wat::core::quote
                (:wat::holon::Bind
                  (:wat::holon::Atom "k")
                  (:wat::holon::Atom "v")))
              0
              :my::test::count-visit) -> :wat::core::i64
            ((:wat::core::Ok pair)
              (:wat::core::second pair))
            ((:wat::core::Err _e) -1)))
    "#;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 1, "expected count=1; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

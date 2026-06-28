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

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

/// `:wat::eval::WalkStep<A>` (the first parametric built-in enum).
/// A function whose body returns `(:wat::eval::WalkStep::Continue
/// <i64>)` must satisfy a `-> :wat::eval::WalkStep<wat::core::i64>` signature.
/// Pre-arc-071 this failed type-check because the synthesized
/// constructor's return type was bare `:wat::eval::WalkStep`.
/// Arc 170 slice 1f-ζ: :my::compute calls :my::test::wrap and returns i64.
#[test]
fn walkstep_continue_parametric_inference_at_use_site() {
    assert!(matches!(run("tests/types/parametric_enum_walkstep_continue.wat"), Value::i64(7)), "expected i64(7)");
}

#[test]
fn walkstep_skip_parametric_inference_at_use_site() {
    // `Skip` takes (terminal :HolonAST, acc :A). Same parametric
    // inference path but with a different field count.
    // Arc 170 slice 1f-ζ: :my::compute calls :my::test::halt and returns i64.
    assert!(matches!(run("tests/types/parametric_enum_walkstep_skip.wat"), Value::i64(3)), "expected i64(3)");
}

/// The full walker pattern from arc 070's USER-GUIDE example,
/// frozen + type-checked. Equivalent to the lab harness probe at
/// `holon-lab-trading/wat-tests-integ/experiment/099-walkstep-probe`
/// — pre-arc-071, both this test and that probe failed; post-fix,
/// both pass.
/// Arc 170 slice 1f-ζ: :my::compute runs the walk and returns the count.
#[test]
fn walk_visitor_signature_matches_at_use_site() {
    match run("tests/types/parametric_enum_walk_visitor.wat") {
        Value::i64(n) => assert_eq!(n, 1, "expected count=1; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

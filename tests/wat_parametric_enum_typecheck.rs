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

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// `:wat::eval::WalkStep<A>` (the first parametric built-in enum).
/// A function whose body returns `(:wat::eval::WalkStep::Continue
/// <i64>)` must satisfy a `-> :wat::eval::WalkStep<wat::core::i64>` signature.
/// Pre-arc-071 this failed type-check because the synthesized
/// constructor's return type was bare `:wat::eval::WalkStep`.
#[test]
fn walkstep_continue_parametric_inference_at_use_site() {
    let src = r#"
        (:wat::core::define
          (:my::test::wrap (n :wat::core::i64) -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Continue n))
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((wrapped :wat::eval::WalkStep<wat::core::i64>) (:my::test::wrap 7)))
            (:wat::io::IOWriter/println stdout "ok")))
    "#;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

#[test]
fn walkstep_skip_parametric_inference_at_use_site() {
    // `Skip` takes (terminal :HolonAST, acc :A). Same parametric
    // inference path but with a different field count.
    let src = r#"
        (:wat::core::define
          (:my::test::halt
            (n :wat::core::i64)
            -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Skip
            (:wat::holon::leaf 999)
            n))
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((halted :wat::eval::WalkStep<wat::core::i64>) (:my::test::halt 3)))
            (:wat::io::IOWriter/println stdout "ok")))
    "#;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

/// The full walker pattern from arc 070's USER-GUIDE example,
/// frozen + type-checked. Equivalent to the lab harness probe at
/// `holon-lab-trading/wat-tests-integ/experiment/099-walkstep-probe`
/// — pre-arc-071, both this test and that probe failed; post-fix,
/// both pass.
#[test]
fn walk_visitor_signature_matches_at_use_site() {
    let src = r#"
        (:wat::core::define
          (:my::test::count-visit
            (acc :wat::core::i64)
            (form :wat::WatAST)
            (step :wat::eval::StepResult)
            -> :wat::eval::WalkStep<wat::core::i64>)
          (:wat::eval::WalkStep::Continue (:wat::core::i64::+,2 acc 1)))
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::match
            (:wat::eval::walk
              (:wat::core::quote
                (:wat::holon::Bind
                  (:wat::holon::Atom "k")
                  (:wat::holon::Atom "v")))
              0
              :my::test::count-visit) -> :wat::core::unit
            ((:wat::core::Ok pair)
              (:wat::core::let*
                (((count :wat::core::i64) (:wat::core::second pair)))
                (:wat::core::if (:wat::core::= count 1) -> :wat::core::unit
                  (:wat::io::IOWriter/println stdout "ok")
                  (:wat::io::IOWriter/println stdout "wrong-count"))))
            ((:wat::core::Err _e) (:wat::io::IOWriter/println stdout "walk-err"))))
    "#;
    assert_eq!(run(src), vec!["ok".to_string()]);
}

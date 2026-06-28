//! Arc 170 slice 3 Gap K — spawn-process stdout-capture verification.
//!
//! Verifies that `run-hermetic-ast` (the spawn-process Layer 2 path) can
//! capture stdout written by the child program, and that the drain-before-join
//! shape in `run-sandboxed-hermetic-ast` does not lose the captured output.
//!
//! ## Path exercised
//!
//! This file uses `:wat::test::run-hermetic-ast` exclusively — the Layer 2
//! spawn-process surface. This path forks the current process (COW clone)
//! and builds a fresh FrozenWorld from the provided AST forms. The forked
//! child inherits the parent's ambient stdio services; `(:wat::kernel::println ...)`
//! writes to the child's ambient stdout, which the substrate captures in the
//! `Process/stdout` IOReader pipe. The drain-before-join shape in
//! `run-sandboxed-hermetic-ast` (fixed in Gap K) ensures the stdout IOReader
//! is drained BEFORE `Process/join-result` is called.
//!
//! ## Row C2 + Row G verification
//!
//! File name: `probe_run_hermetic_ast_stdout_capture.rs`
//! Surface exercised: `:wat::test::run-hermetic-ast` (spawn-process)
//! These match. Every probe body in this file exercises the spawn-process path.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// ─── Probe 1 — fork-program-ast child writes stdout; parent captures it ────

/// `run-hermetic-ast` with a child program that calls
/// `(:wat::kernel::println "hello-from-probe")`.
///
/// The child uses `fork-program-ast` (COW fork) and installs the ambient
/// stdio services before running `:user::main`. The `println` call writes
/// to the child's stdout pipe. The parent's drain-before-join in
/// `run-sandboxed-hermetic-ast` (fixed in Gap K) reads the IOReader obtained
/// from `Process/stdout` BEFORE calling `Process/join-result`.
///
/// Verifies:
/// - `RunResult.stdout` contains the line "hello-from-probe"
/// - `RunResult.failure = None` (child exited cleanly)
///
/// Path: `:wat::test::run-hermetic-ast` (fork-program-ast Layer 2).
#[test]
fn probe_run_hermetic_ast_child_stdout_captured() {
    // World loaded from co-located probe_run_hermetic_ast_stdout_capture.wat via startup_beside.
    let world = startup_beside(file!()).expect("freeze should succeed");

    // Evaluate (:probe::ast::capture-stdout) to get the RunResult.
    let call = wat::parse_one!("(:probe::ast::capture-stdout)").expect("parse call");
    let env = Environment::new();
    let result = eval_in_frozen(&call, &world, &env)
        .expect("probe::ast::capture-stdout should run without panicking").value_owned();

    // result is :wat::kernel::RunResult { stdout stderr failure }
    let sv = match &result {
        Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::RunResult" => s,
        other => panic!("expected RunResult Struct; got {:?}", other),
    };

    // RunResult field 0 is stdout :Vector<String>.
    let stdout_lines = match &sv.fields[0] {
        Value::Vec(v) => v.as_ref(),
        other => panic!("expected Vec stdout field; got {:?}", other),
    };
    assert!(
        !stdout_lines.is_empty(),
        "expected at least one stdout line (child called println); got empty stdout"
    );
    // The println call writes "hello-from-probe" (println EDN-serializes
    // strings; a String value writes as quoted "hello-from-probe" or
    // unquoted hello-from-probe depending on the printer path).
    // We assert the first line contains the sentinel text.
    let first_line = match &stdout_lines[0] {
        Value::String(s) => s.to_string(),
        other => panic!("expected String stdout line; got {:?}", other),
    };
    assert!(
        first_line.contains("hello-from-probe"),
        "expected stdout line to contain 'hello-from-probe'; got: {:?}",
        first_line
    );

    // RunResult field 2 is failure :Option<Failure>; must be None (clean exit).
    let failure_field = &sv.fields[2];
    let is_none = match failure_field {
        Value::Option(opt) => opt.as_ref().is_none(),
        other => panic!("expected Option failure field; got {:?}", other),
    };
    assert!(
        is_none,
        "expected child with println to exit cleanly (failure=None); got {:?}",
        result
    );
}

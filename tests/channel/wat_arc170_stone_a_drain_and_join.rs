//! Arc 170 Stone A — `:wat::kernel::Thread/drain-and-join` +
//! `:wat::kernel::Process/drain-and-join` substrate primitives.
//!
//! These tests prove the Stone A surface end-to-end:
//!
//! 1. `Thread/drain-and-join` on a clean-exiting thread returns
//!    `Ok(())` after draining its output channel.
//! 2. `Process/drain-and-join` on a clean-exiting process returns
//!    `Ok(())` after draining stdout + stderr to EOF.
//! 3. `Thread/drain-and-join` on a panicking thread returns
//!    `Err(chain)` carrying a `ThreadDiedError::Panic` head.
//! 4. `Process/drain-and-join` on a panicking process returns
//!    `Err(chain)` carrying a `ProcessDiedError` head.
//!
//! The drain step is the discipline this stone embodies in the
//! substrate (rather than in `-with-io` driver code) — pulling all
//! buffered output before joining prevents the lockstep deadlock
//! arc 117/133's walker machinery currently guards against.

use wat::ast::WatAST;
use wat::freeze::{startup_bare, startup_beside};
use wat::runtime::{eval, Environment, Value};

// ─── helpers ───────────────────────────────────────────────────────────

/// Read a co-located spawn-process child program (a separate subprocess source, not the parent world).
fn read_child(name: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/channel")
        .join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read child fixture {p:?}: {e}"))
}

/// Helper to build a `(:wat::kernel::spawn-process (:wat::core::forms ...))`
/// call AST from a child-program source. Mirrors the helper in
/// `wat_arc170_program_contracts.rs`.
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items =
        vec![WatAST::Keyword(":wat::core::forms".into(), wat::rust_caller_span!())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, wat::rust_caller_span!());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::rust_caller_span!()),
            forms_call,
        ],
        wat::rust_caller_span!(),
    )
}

/// Unwrap `Value::Result(Ok(_))` and assert the Ok payload is unit
/// (nil). The wrapper return shape for both Thread/drain-and-join and
/// Process/drain-and-join is `Result<(), Vec<*DiedError>>`, with the
/// Ok arm carrying `Value::Unit` on clean exit.
fn assert_result_ok_unit(v: &Value, label: &str) {
    match v {
        Value::Result(r) => match r.as_ref() {
            Ok(Value::Unit) => {}
            Ok(other) => panic!("{}: expected Ok(()); got Ok({:?})", label, other),
            Err(e) => panic!("{}: expected Ok(()); got Err({:?})", label, e),
        },
        other => panic!("{}: expected Value::Result; got {:?}", label, other),
    }
}

/// Unwrap `Value::Result(Err(_))` and return the Err payload (the
/// Vec<*DiedError> chain) for further inspection. Panics on the Ok
/// arm or any non-Result value.
fn unwrap_result_err<'a>(v: &'a Value, label: &str) -> &'a Value {
    match v {
        Value::Result(r) => match r.as_ref() {
            Err(e) => e,
            Ok(other) => panic!("{}: expected Err; got Ok({:?})", label, other),
        },
        other => panic!("{}: expected Value::Result; got {:?}", label, other),
    }
}

// ─── Stone A T1. Thread/drain-and-join — clean exit returns Ok(()) ────

#[test]
fn stone_a_thread_drain_and_join_clean_exit_returns_ok() {
    // The worker thread sends three i64 values to its output Sender,
    // then returns nil. The PARENT does NOT recv any of them; instead
    // Thread/drain-and-join is responsible for draining the output
    // channel before joining. A clean exit yields Ok(()).
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":my::test::drain-thread")
        .expect(":my::test::drain-thread defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("Thread/drain-and-join should succeed");
    assert_result_ok_unit(&outcome, "Thread/drain-and-join clean exit");
}

// ─── Stone A T2. Process/drain-and-join — clean exit returns Ok(()) ───

#[test]
fn stone_a_process_drain_and_join_clean_exit_returns_ok() {
    // The child process prints three lines to stdout (arc 278: the former
    // incidental stderr `eprintln "diag"` migrated to `println` — eprintln is
    // now terminal and would crash the child), then exits clean (nil return →
    // exit code 0). The parent does NOT read stdout/stderr; Process/drain-and-
    // join is responsible for draining both pipes before joining (stderr drains
    // empty-to-EOF). A clean exit yields Ok(()).
    let world = startup_bare().expect("startup");
    let child = read_child("wat_arc170_stone_a_drain_and_join_child_clean.wat");
    let call = build_spawn_process_call(&child);
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    // Rebind into a child env so we can reference the Process struct by
    // name from a hand-built drain-and-join AST.
    let env2 = Environment::new().child().bind("proc", wat::rust_caller_span!(), process.into()).build();
    let call_djoin = wat::parse_one!("(:wat::kernel::Process/drain-and-join proc)")
        .expect("drain-and-join AST parses");
    let outcome = eval(&call_djoin, &env2, world.symbols())
        .expect("Process/drain-and-join should succeed").value_owned();
    assert_result_ok_unit(&outcome, "Process/drain-and-join clean exit");
}

// ─── Stone A T3. Thread/drain-and-join — panic returns Err(chain) ─────

#[test]
fn stone_a_thread_drain_and_join_panic_returns_err() {
    // The worker thread panics via Option/expect on None. The drain
    // pass should still complete (recv-until-Disconnected sees the
    // sender drop from the panicked thread), then the inner join
    // returns Err with a ThreadDiedError::Panic head.
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(":my::test::drain-panicking-thread")
        .expect(":my::test::drain-panicking-thread defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("Thread/drain-and-join should return Result (not Rust-panic)");
    let chain = unwrap_result_err(&outcome, "Thread/drain-and-join panic");
    // The chain is a Vec of ThreadDiedError; head should be a Panic
    // variant. We only check that the chain is non-empty here (full
    // panic-message assertions belong in arc 113 tests).
    match chain {
        Value::Vec(v) => assert!(
            !v.is_empty(),
            "expected non-empty died-chain; got empty"
        ),
        other => panic!(
            "Thread/drain-and-join panic: expected Vec of ThreadDiedError; got {:?}",
            other
        ),
    }
}

// ─── Stone A T4. Process/drain-and-join — panic returns Err(chain) ────

#[test]
fn stone_a_process_drain_and_join_panic_returns_err() {
    // The child process panics intentionally before exiting. Substrate
    // cascades the panic chain through stderr; child exits non-zero;
    // drain-and-join's drain pass consumes stdout + stderr to EOF, then
    // join surfaces the non-zero exit code as Err(chain).
    let world = startup_bare().expect("startup");
    let child = read_child("wat_arc170_stone_a_drain_and_join_child_panic.wat");
    let call = build_spawn_process_call(&child);
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let env2 = Environment::new().child().bind("proc", wat::rust_caller_span!(), process.into()).build();
    let call_djoin = wat::parse_one!("(:wat::kernel::Process/drain-and-join proc)")
        .expect("drain-and-join AST parses");
    let outcome = eval(&call_djoin, &env2, world.symbols())
        .expect("Process/drain-and-join should return Result (not Rust-panic)").value_owned();
    let chain = unwrap_result_err(&outcome, "Process/drain-and-join panic");
    match chain {
        Value::Vec(v) => assert!(
            !v.is_empty(),
            "expected non-empty died-chain; got empty"
        ),
        other => panic!(
            "Process/drain-and-join panic: expected Vec of ProcessDiedError; got {:?}",
            other
        ),
    }
}

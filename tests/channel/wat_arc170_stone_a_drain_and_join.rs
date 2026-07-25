//! Arc 170 Stone A — `:wat::kernel::Thread/drain-and-join` (thread tier) +
//! the process-tier peer drain. Arc 278 IPC de-prime: the process cases now
//! spawn a `spawn-program' (process)` peer and drain it via `:wat::kernel::recv-all'`
//! (the honest peer-drain that replaces the 4-field Process's `Process/drain-and-join`).
//!
//! These tests prove the Stone A surface end-to-end:
//!
//! 1. `Thread/drain-and-join` on a clean-exiting thread returns
//!    `Ok(())` after draining its output channel.
//! 2. `recv-all'` on a clean-exiting `spawn-program' (process)` peer returns
//!    `Ok(Vector<..>)` (collected outputs) after the peer Closes (arc 278 IPC de-prime).
//! 3. `Thread/drain-and-join` on a panicking thread returns
//!    `Err(chain)` carrying a `ThreadDiedError::Panic` head.
//! 4. `recv-all'` on a panicking `spawn-program' (process)` peer returns
//!    `Err(cause)` carrying the `LociDiedError` from the peer's RecvOutcome::Lost.
//!
//! The drain step is the discipline this stone embodies in the
//! substrate (rather than in `-with-io` driver code) — pulling all
//! buffered output before joining prevents the lockstep deadlock
//! arc 117/133's walker machinery currently guards against.

use wat::ast::WatAST;
use wat::freeze::startup_beside;
use wat::runtime::{eval, Environment, Value};

// ─── helpers ───────────────────────────────────────────────────────────

/// Read a co-located spawn-program' (process) child program (a separate subprocess source, not the parent world).
fn read_child(name: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/channel")
        .join(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read child fixture {p:?}: {e}"))
}

/// Helper to build a `(:wat::kernel::spawn-program' (:wat::spawn::process)
/// (:wat::core::forms ...))` call AST from a child-program source (arc 278 IPC
/// de-prime — the process peer replaces the 4-field Process). Mirrors the primed
/// spawn shape proven in tests/kernel/wat_hermetic_round_trip.wat.
fn build_spawn_program_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-program-process>")
            .expect("child program parse");
    let mut forms_items =
        vec![WatAST::Keyword(":wat::core::forms".into(), wat::rust_caller_span!())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, wat::rust_caller_span!());
    // The process locus key: `(:wat::spawn::process)`.
    let process_locus = WatAST::List(
        vec![WatAST::Keyword(":wat::spawn::process".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program'".into(), wat::rust_caller_span!()),
            process_locus,
            forms_call,
        ],
        wat::rust_caller_span!(),
    )
}

/// Unwrap `Value::Result(Ok(_))` and assert the Ok payload is unit
/// (nil). The thread-tier `Thread/drain-and-join` return shape is
/// `Result<(), Vec<*DiedError>>`, with the Ok arm carrying `Value::Unit`
/// on clean exit. (The process tier now drains via `recv-all'`, whose Ok
/// arm carries a `Vector<O>` — see `assert_result_ok_any`.)
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

/// Unwrap `Value::Result(Ok(_))` with ANY Ok payload. The process-tier drain is
/// now `recv-all'`, whose Ok arm carries the collected `Vector<O>` (not the unit
/// `()` the thread-tier `Thread/drain-and-join` returns) — so the clean-exit
/// contract here is "Ok, whatever it collected", not "Ok(())".
fn assert_result_ok_any(v: &Value, label: &str) {
    match v {
        Value::Result(r) => match r.as_ref() {
            Ok(_) => {}
            Err(e) => panic!("{}: expected Ok; got Err({:?})", label, e),
        },
        other => panic!("{}: expected Value::Result; got {:?}", label, other),
    }
}

/// Unwrap `Value::Result(Err(_))` and return the Err payload for further
/// inspection. For the thread tier this is the `Vec<*DiedError>` chain; for the
/// process tier `recv-all'` it is a single structured `LociDiedError`. Panics on
/// the Ok arm or any non-Result value.
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

// ─── Stone A T2. recv-all' over a process peer — clean exit returns Ok ─

#[test]
fn stone_a_process_drain_and_join_clean_exit_returns_ok() {
    // Arc 278 IPC de-prime: the child process is now a `spawn-program' (process)`
    // peer. Its `:user::main` `println`s three lines (each arrives at the parent as
    // a RecvOutcome::Message), then exits clean (nil return → the peer Closes). The
    // parent drains the peer via `recv-all'`, which collects the outputs and returns
    // Ok(Vector<String>) on the clean Close.
    // just-eval (rubric): the spawn is built Rust-side (WatAST nodes, not a parsed string —
    // never trips no_inlined_wat); the drain call is the co-located fixture's
    // `:my::test::drain-process`, applied with the spawned Process' peer as its argument.
    let world = startup_beside(file!()).expect("startup");
    let child = read_child("wat_arc170_stone_a_drain_and_join_child_clean.wat");
    let call = build_spawn_program_process_call(&child);
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-program' (process) succeeds").value_owned();
    let func = world
        .symbols()
        .get(":my::test::drain-process")
        .expect(":my::test::drain-process defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        vec![process],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("recv-all' drain should succeed");
    // Clean exit → recv-all' returns Ok(collected outputs) after the peer Closes.
    assert_result_ok_any(&outcome, "recv-all' clean exit");
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

// ─── Stone A T4. recv-all' over a process peer — panic returns Err ────

#[test]
fn stone_a_process_drain_and_join_panic_returns_err() {
    // Arc 278 IPC de-prime: the child `spawn-program' (process)` peer panics
    // intentionally before printing anything. The parent's first `recv'` (inside
    // `recv-all'`) sees the peer DIE — RecvOutcome::Lost — so `recv-all'` returns
    // Err(cause), the LociDiedError carrying the child's death reason.
    let world = startup_beside(file!()).expect("startup");
    let child = read_child("wat_arc170_stone_a_drain_and_join_child_panic.wat");
    let call = build_spawn_program_process_call(&child);
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-program' (process) succeeds").value_owned();
    let func = world
        .symbols()
        .get(":my::test::drain-process")
        .expect(":my::test::drain-process defined");
    let outcome = wat::runtime::apply_function(
        func.clone(),
        vec![process],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("recv-all' drain should return Result (not Rust-panic)");
    // The peer DIED (child panicked) → recv-all' surfaces RecvOutcome::Lost as
    // Err(cause), where cause is a single structured `LociDiedError` (NOT the
    // Vec<*DiedError> chain the thread-tier drain-and-join returns). The contract
    // here is simply that the death is surfaced as the Err arm, never swallowed;
    // unwrap_result_err asserts exactly that.
    let _cause = unwrap_result_err(&outcome, "recv-all' panic");
}

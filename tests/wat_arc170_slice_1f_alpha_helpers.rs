//! Arc 170 slice 1f-α — substrate primitives `:wat::kernel::println`,
//! `:wat::kernel::eprintln`, `:wat::kernel::readln`.
//!
//! These three primitives look up per-thread channel handles from
//! a thread-local [`wat::thread_io::ThreadIO`] cell and run the
//! mini-TCP block-on-completion lockstep. Slice 1f-α delivers the
//! substrate side; slices 1f-β / γ / δ ship the wat-side service
//! implementations + orchestrator + boot wiring.
//!
//! The 10 rows in this fixture cover:
//!
//! | Row | Test | Concern |
//! |-----|------|---------|
//! | A | unpopulated println | clean ServiceNotRunning, no panic |
//! | B | unpopulated eprintln | same shape |
//! | C | unpopulated readln | same shape |
//! | D | populated println sends serialized String | round-trip |
//! | E | populated eprintln sends serialized String | round-trip via stderr pair |
//! | F | populated readln returns received form | reverse direction |
//! | G | polymorphic value types — i64 / String / bool / tuple / struct | value_to_edn coverage |
//! | H | type-check accepts any-T for println | scheme registration |
//! | I | type-check accepts any-T for eprintln | scheme registration |
//! | J | type-check infers HolonAST return for readln | scheme registration |
//!
//! ThreadIO is per-thread; cargo's test-runner reuses worker
//! threads, so every populated row calls `uninstall_thread_io` on
//! exit to keep the cell clean between tests.

use std::sync::Arc;

use crossbeam_channel::bounded;
use holon::HolonAST;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, RuntimeError, Value};
use wat::thread_io::{
    install_thread_io, uninstall_thread_io, ThreadIO,
    StdInServiceEvent, StdOutServiceEvent, StdErrServiceEvent,
};

// ─── helpers ───────────────────────────────────────────────────────

/// Build a frozen world that contains a no-op `:user::main`. The
/// invocation tests evaluate ad-hoc forms via `eval_in_frozen` so
/// the substrate's freeze pipeline runs (registering the type-check
/// arms + dispatch) without needing a meaningful main body.
fn freeze_skeleton() -> wat::freeze::FrozenWorld {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("skeleton freeze succeeds")
}

/// Build a [`ThreadIO`] for tests. Returns the IO + the service-side
/// channel ends so the test can drive the service-side conversation
/// from a tester thread. Channel ends carry Event variants per
/// pass 18 — service-side receives an Event and matches the variant.
struct TestRig {
    io: Option<ThreadIO>,
    /// service-side: receive StdOutServiceEvent from println.
    out_rx: crossbeam_channel::Receiver<StdOutServiceEvent>,
    out_ack_tx: crossbeam_channel::Sender<()>,
    /// service-side: receive StdErrServiceEvent from eprintln.
    err_rx: crossbeam_channel::Receiver<StdErrServiceEvent>,
    err_ack_tx: crossbeam_channel::Sender<()>,
    /// service-side: receive StdInServiceEvent from readln.
    stdin_rx: crossbeam_channel::Receiver<StdInServiceEvent>,
    stdin_reply_tx: crossbeam_channel::Sender<Arc<HolonAST>>,
}

fn build_rig() -> TestRig {
    let (out_tx, out_rx) = bounded::<StdOutServiceEvent>(1);
    let (out_ack_tx, out_ack_rx) = bounded::<()>(1);
    let (err_tx, err_rx) = bounded::<StdErrServiceEvent>(1);
    let (err_ack_tx, err_ack_rx) = bounded::<()>(1);
    let (stdin_tx, stdin_rx) = bounded::<StdInServiceEvent>(1);
    let (stdin_reply_tx, stdin_reply_rx) = bounded::<Arc<HolonAST>>(1);

    let io = ThreadIO {
        stdout_tx: out_tx,
        stdout_ack_rx: out_ack_rx,
        stderr_tx: err_tx,
        stderr_ack_rx: err_ack_rx,
        stdin_tx,
        stdin_reply_rx,
    };

    TestRig {
        io: Some(io),
        out_rx,
        out_ack_tx,
        err_rx,
        err_ack_tx,
        stdin_rx,
        stdin_reply_tx,
    }
}

/// Install the rig's ThreadIO into the calling thread's cell, run
/// `body`, drain on exit. Cargo reuses test threads so leaking
/// ThreadIO across tests would break isolation.
fn run_with_thread_io<F, T>(rig: &mut TestRig, body: F) -> T
where
    F: FnOnce() -> T,
{
    let io = rig.io.take().expect("ThreadIO consumed twice");
    install_thread_io(io);
    let result = body();
    let _ = uninstall_thread_io();
    result
}

/// Drain any leftover ThreadIO from the calling thread before the
/// row's body runs. Cargo's worker threads are reused; an earlier
/// row that panicked between install + uninstall would otherwise
/// leak its IO into this row.
fn fresh_thread() {
    let _ = uninstall_thread_io();
}

// ─── A. unpopulated println ────────────────────────────────────────

#[test]
fn row_a_println_unpopulated_returns_service_not_running() {
    fresh_thread();
    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::println 42)").expect("parse println form");
    let env = Environment::new();
    let err = eval_in_frozen(&ast, &world, &env)
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError::ServiceNotRunning { op, .. } => {
            assert_eq!(op, ":wat::kernel::println");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── B. unpopulated eprintln ───────────────────────────────────────

#[test]
fn row_b_eprintln_unpopulated_returns_service_not_running() {
    fresh_thread();
    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::eprintln 42)").expect("parse eprintln form");
    let env = Environment::new();
    let err = eval_in_frozen(&ast, &world, &env)
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError::ServiceNotRunning { op, .. } => {
            assert_eq!(op, ":wat::kernel::eprintln");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── C. unpopulated readln ─────────────────────────────────────────

#[test]
fn row_c_readln_unpopulated_returns_service_not_running() {
    fresh_thread();
    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::readln)").expect("parse readln form");
    let env = Environment::new();
    let err = eval_in_frozen(&ast, &world, &env)
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError::ServiceNotRunning { op, .. } => {
            assert_eq!(op, ":wat::kernel::readln");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── D. populated println sends serialized String ──────────────────

#[test]
fn row_d_println_populated_sends_serialized_string() {
    fresh_thread();
    let mut rig = build_rig();
    // Tester thread plays "service" — receives the Write event, extracts
    // the line, immediately acks.
    let out_rx = rig.out_rx.clone();
    let out_ack_tx = rig.out_ack_tx.clone();
    let tester = std::thread::spawn(move || {
        let event = out_rx.recv().expect("service receives event");
        let line = match event {
            StdOutServiceEvent::Write { line } => line,
            _ => panic!("expected Write variant"),
        };
        out_ack_tx.send(()).expect("service acks");
        line
    });

    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::println 42)").expect("parse println form");
    let env = Environment::new();
    let result = run_with_thread_io(&mut rig, || eval_in_frozen(&ast, &world, &env));

    assert!(matches!(result, Ok(Value::Unit)), "got {:?}", result);
    let received = tester.join().expect("tester joins");
    assert_eq!(received, "42");
}

// ─── E. populated eprintln sends serialized String ─────────────────

#[test]
fn row_e_eprintln_populated_sends_serialized_string() {
    fresh_thread();
    let mut rig = build_rig();
    let err_rx = rig.err_rx.clone();
    let err_ack_tx = rig.err_ack_tx.clone();
    let tester = std::thread::spawn(move || {
        let event = err_rx.recv().expect("service receives event");
        let line = match event {
            StdErrServiceEvent::Write { line } => line,
            _ => panic!("expected Write variant"),
        };
        err_ack_tx.send(()).expect("service acks");
        line
    });

    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::eprintln \"hello\")").expect("parse eprintln form");
    let env = Environment::new();
    let result = run_with_thread_io(&mut rig, || eval_in_frozen(&ast, &world, &env));

    assert!(matches!(result, Ok(Value::Unit)), "got {:?}", result);
    let received = tester.join().expect("tester joins");
    // EDN-quoted: a wat String renders as "\"hello\"".
    assert_eq!(received, "\"hello\"");
}

// ─── F. populated readln returns received form ─────────────────────

#[test]
fn row_f_readln_populated_returns_received_form() {
    fresh_thread();
    let mut rig = build_rig();
    // Build a small HolonAST to hand back to the substrate. A
    // String leaf is the simplest cell.
    let expected_ast = Arc::new(HolonAST::String(Arc::from("ok")));

    let stdin_rx = rig.stdin_rx.clone();
    let stdin_reply_tx = rig.stdin_reply_tx.clone();
    let payload = Arc::clone(&expected_ast);
    let tester = std::thread::spawn(move || {
        let event = stdin_rx.recv().expect("service receives event");
        match event {
            StdInServiceEvent::Read => {}
            _ => panic!("expected Read variant"),
        }
        stdin_reply_tx.send(payload).expect("service sends reply");
    });

    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::readln)").expect("parse readln form");
    let env = Environment::new();
    let result = run_with_thread_io(&mut rig, || eval_in_frozen(&ast, &world, &env));

    tester.join().expect("tester joins");
    match result {
        Ok(Value::holon__HolonAST(got)) => {
            assert_eq!(*got, *expected_ast, "readln returned the AST the service sent");
        }
        other => panic!("expected Value::holon__HolonAST; got {:?}", other),
    }
}

// ─── G. polymorphic value types serialize correctly ────────────────

#[test]
fn row_g_println_polymorphic_value_types() {
    // Each row exercises println with a different wat value type
    // and asserts the EDN serialization matches what
    // value_to_edn_with produces. The substrate decides what each
    // primitive renders as; this test pins that contract for the
    // common scalar shapes.
    let cases: &[(&str, &str)] = &[
        ("(:wat::kernel::println 42)", "42"),
        ("(:wat::kernel::println \"hello\")", "\"hello\""),
        ("(:wat::kernel::println true)", "true"),
        ("(:wat::kernel::println false)", "false"),
        // A 2-tuple — value_to_edn renders Tuples as Vectors.
        // `:wat::core::Tuple` is the verb-equals-type constructor
        // (arc 109 slice 1g). The runtime produces a Value::Tuple
        // which value_to_edn maps to an EDN Vector.
        (
            "(:wat::kernel::println (:wat::core::Tuple 1 2))",
            "[1 2]",
        ),
    ];

    for (src, expected) in cases {
        fresh_thread();
        let mut rig = build_rig();
        let out_rx = rig.out_rx.clone();
        let out_ack_tx = rig.out_ack_tx.clone();
        let tester = std::thread::spawn(move || {
            let event = out_rx.recv().expect("service receives event");
            let line = match event {
                StdOutServiceEvent::Write { line } => line,
                _ => panic!("expected Write variant"),
            };
            out_ack_tx.send(()).expect("service acks");
            line
        });

        let world = freeze_skeleton();
        let ast = wat::parse_one!(src).expect("parse polymorphic form");
        let env = Environment::new();
        let result = run_with_thread_io(&mut rig, || eval_in_frozen(&ast, &world, &env));

        assert!(matches!(result, Ok(Value::Unit)), "src={:?} got {:?}", src, result);
        let received = tester.join().expect("tester joins");
        assert_eq!(received, *expected, "src={:?}", src);
    }
}

// ─── H. type-check accepts any-T for println ───────────────────────

#[test]
fn row_h_type_check_println_accepts_any_t() {
    // If println's type scheme is ∀T. T -> :wat::core::nil, freezing
    // a `:test::p` define that returns nil after calling println on
    // an i64 must succeed. Failure surfaces as a freeze error;
    // success means the type-check arm registered correctly.
    let src = r#"
        (:wat::core::define (:test::p (v :wat::core::i64) -> :wat::core::nil)
          (:wat::kernel::println v))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "println should type-check against any-T input; got: {:?}",
        result.err()
    );
}

// ─── I. type-check accepts any-T for eprintln ──────────────────────

#[test]
fn row_i_type_check_eprintln_accepts_any_t() {
    let src = r#"
        (:wat::core::define (:test::p (v :wat::core::String) -> :wat::core::nil)
          (:wat::kernel::eprintln v))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "eprintln should type-check against any-T input; got: {:?}",
        result.err()
    );
}

// ─── J. type-check infers HolonAST return for readln ───────────────

#[test]
fn row_j_type_check_readln_returns_holonast() {
    // `:test::r` declares its return as :wat::holon::HolonAST and its
    // body is exactly `(:wat::kernel::readln)`. Successful freeze
    // proves the return type unifies — the scheme says
    // `() -> :wat::holon::HolonAST`.
    let src = r#"
        (:wat::core::define (:test::r -> :wat::holon::HolonAST)
          (:wat::kernel::readln))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "readln return type should unify with :wat::holon::HolonAST; got: {:?}",
        result.err()
    );
}

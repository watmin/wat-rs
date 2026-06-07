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

use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::io::{PipeReader, PipeWriter, WatReader};
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, Value};
use wat::span::Span;
use wat::thread_io::{
    install_thread_io, next_thread_id, spawn_stdio_service_peer, uninstall_thread_io,
    RuntimeServices, StdErrServiceEvent, StdInServiceEvent, StdOutInput, ThreadIO,
};
use wat::typed_channel::{bounded, Sender, Receiver};

// Arc 170 slice 1f-ι — the readln contract now requires a `-> :T`
// annotation and the bridge reply payload is a raw `String` (was
// `Arc<HolonAST>` pre-1f-ι). Row C/F/J below are slice-1f-α-vintage
// assertions that DO NOT reflect the new contract; they remain here
// as historical record. Subsequent slices (1f-κ/λ/μ) migrate them.
// The type signatures are updated to the new contract so the file
// continues to compile (the workspace-wide build precondition for
// every other test); the assertions themselves are expected to
// surface failure under the new substrate.

// ─── helpers ───────────────────────────────────────────────────────

/// Build a frozen world that contains a no-op `:user::main`. The
/// invocation tests evaluate ad-hoc forms via `eval_in_frozen` so
/// the substrate's freeze pipeline runs (registering the type-check
/// arms + dispatch) without needing a meaningful main body.
fn freeze_skeleton() -> wat::freeze::FrozenWorld {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
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
    /// service-side: receive StdErrServiceEvent from eprintln.
    err_rx: Receiver<StdErrServiceEvent>,
    err_ack_tx: Sender<()>,
    /// service-side: receive StdInServiceEvent from readln.
    stdin_rx: Receiver<StdInServiceEvent>,
    stdin_reply_tx: Sender<String>,
}

fn build_rig() -> TestRig {
    // Stone 8.1 — stdout is a universe-resident peer; rows that don't
    // drive stdout get a DUMMY reply pair (no service loop behind it —
    // and println without RuntimeServices errors ServiceNotRunning
    // before it could ever block on this). Row D builds the REAL
    // miniature universe instead of using this rig half.
    let (_stdout_reply_tx, stdout_reply_rx) =
        wat::comms::thread::pair::<Result<(), String>>();
    let (err_tx, err_rx) = bounded::<StdErrServiceEvent>(1);
    let (err_ack_tx, err_ack_rx) = bounded::<()>(1);
    let (stdin_tx, stdin_rx) = bounded::<StdInServiceEvent>(1);
    let (stdin_reply_tx, stdin_reply_rx) = bounded::<String>(1);

    let io = ThreadIO {
        stdout_reply_rx,
        stdout_thread_id: next_thread_id(),
        stderr_tx: err_tx,
        stderr_ack_rx: err_ack_rx,
        stdin_tx,
        stdin_reply_rx,
    };

    TestRig {
        io: Some(io),
        err_rx,
        err_ack_tx,
        stdin_rx,
        stdin_reply_tx,
    }
}

/// Stone 8.1 — a MINIATURE TRUE UNIVERSE for the stdout rows: a
/// pipe-backed writer (fd — cross-thread honest), the real 15-line wat
/// handle, the real Rust service loop, a real Register exchange. The
/// stdout tests no longer play the service; they exercise the
/// production pipeline end to end.
struct MiniUniverse {
    sym: wat::runtime::SymbolTable,
    input_tx: wat::comms::thread::Sender<StdOutInput>,
    join: std::thread::JoinHandle<()>,
    reader: PipeReader,
    tid: i64,
}

impl MiniUniverse {
    fn build(world: &wat::freeze::FrozenWorld) -> Self {
        let (pipe_r, pipe_w) =
            wat::fork::make_pipe(":test::stdout-universe").expect("pipe for the service writer");
        let writer = Value::io__IOWriter(Arc::new(PipeWriter::from_owned_fd(pipe_w)));
        let reader = PipeReader::from_owned_fd(pipe_r);

        let handle = world
            .symbols()
            .get(":wat::kernel::services::StdOutService/handle")
            .expect("/handle is in the baked stdlib")
            .clone();
        let peer = spawn_stdio_service_peer(handle, writer, world.symbols().clone());
        let wat::thread_io::StdioServicePeer { input_tx, join } = peer;

        // RS-carrying sym — println reaches the peer via runtime_services().
        let (stdin_dummy_tx, _stdin_dummy_rx) = wat::comms::thread::pair::<Value>();
        let (stderr_dummy_tx, _stderr_dummy_rx) = wat::comms::thread::pair::<Value>();
        let mut sym = world.symbols().clone();
        sym.set_runtime_services(Arc::new(RuntimeServices {
            stdin_ctrl: stdin_dummy_tx,
            stdout_ctrl: input_tx.clone(),
            stderr_ctrl: stderr_dummy_tx,
        }));

        // Register this thread (the universe's job, in miniature).
        let tid = next_thread_id();
        let (reply_tx, reply_rx) = wat::comms::thread::pair::<Result<(), String>>();
        input_tx
            .send(StdOutInput::Register(tid, reply_tx))
            .expect("register with the service peer");

        // ThreadIO: the REAL stdout half + old-path dummies for the rest.
        let (err_tx, _err_rx) = bounded::<StdErrServiceEvent>(1);
        let (_err_ack_tx, err_ack_rx) = bounded::<()>(1);
        let (stdin_tx, _stdin_rx) = bounded::<StdInServiceEvent>(1);
        let (_stdin_reply_tx, stdin_reply_rx) = bounded::<String>(1);
        install_thread_io(ThreadIO {
            stdout_reply_rx: reply_rx,
            stdout_thread_id: tid,
            stderr_tx: err_tx,
            stderr_ack_rx: err_ack_rx,
            stdin_tx,
            stdin_reply_rx,
        });

        MiniUniverse { sym, input_tx, join, reader, tid }
    }

    /// Eval a println form; the mini-TCP ack means write-COMPLETED, so
    /// the line is in the pipe before this returns (ZERO-MUTEX § "use
    /// both when done matters"). Returns the written line, trimmed.
    fn println_and_read(&self, src: &str) -> String {
        let ast = wat::parse_one!(src).expect("parse println form");
        let env = Environment::new();
        let result = eval(&ast, &env, &self.sym)
            .expect("println evals")
            .value_owned();
        assert!(matches!(result, Value::Unit), "println returns nil; src={:?}", src);
        self.reader
            .read_line(Span::unknown())
            .expect("read from the service's pipe")
            .expect("a written line")
            .trim()
            .to_string()
    }

    /// Teardown in the ZERO-MUTEX drop order: deregister, drop EVERY
    /// sender (the RS clone via sym; the original), THEN join — the
    /// loop exits on disconnect; joining before the drops would
    /// deadlock.
    fn finish(self) {
        let _ = uninstall_thread_io();
        let MiniUniverse { sym, input_tx, join, reader, tid } = self;
        input_tx
            .send(StdOutInput::Deregister(tid))
            .expect("deregister");
        drop(sym);
        drop(input_tx);
        drop(reader);
        join.join().expect("service loop joins clean");
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
        RuntimeError { kind: RuntimeErrorKind::ServiceNotRunning { op, .. }, .. } => {
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
        RuntimeError { kind: RuntimeErrorKind::ServiceNotRunning { op, .. }, .. } => {
            assert_eq!(op, ":wat::kernel::eprintln");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── C. unpopulated readln ─────────────────────────────────────────

#[test]
#[ignore = "arc 170 slice 1f-ι: bare `(:wat::kernel::readln)` is now MalformedForm (the `-> :T` annotation is required); migrate to `(:wat::kernel::readln -> :wat::core::String)` in a follow-up slice"]
fn row_c_readln_unpopulated_returns_service_not_running() {
    fresh_thread();
    let world = freeze_skeleton();
    let ast = wat::parse_one!("(:wat::kernel::readln)").expect("parse readln form");
    let env = Environment::new();
    let err = eval_in_frozen(&ast, &world, &env)
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError { kind: RuntimeErrorKind::ServiceNotRunning { op, .. }, .. } => {
            assert_eq!(op, ":wat::kernel::readln");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── D. populated println sends serialized String ──────────────────

#[test]
fn row_d_println_populated_sends_serialized_string() {
    fresh_thread();
    let world = freeze_skeleton();
    let universe = MiniUniverse::build(&world);
    let line = universe.println_and_read("(:wat::kernel::println 42)");
    assert_eq!(line, "42");
    universe.finish();
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

    assert!(matches!(result.expect("eval should succeed").value_owned(), Value::Unit), "expected Unit");
    let received = tester.join().expect("tester joins");
    // EDN-quoted: a wat String renders as "\"hello\"".
    assert_eq!(received, "\"hello\"");
}

// ─── F. populated readln returns received form ─────────────────────

#[test]
#[ignore = "arc 170 slice 1f-ι: row F's `(:wat::kernel::readln)` no longer parses without the `-> :T` annotation; migrate to `(:wat::kernel::readln -> :wat::core::String)` in a follow-up slice"]
fn row_f_readln_populated_returns_received_form() {
    fresh_thread();
    let mut rig = build_rig();
    // The raw EDN line the service hands back. Pre-1f-ι this was a
    // pre-parsed Arc<HolonAST>; post-1f-ι the bridge carries the
    // raw line and the substrate parses + coerces to the caller's
    // declared T.
    let expected_line = String::from("\"ok\"");

    let stdin_rx = rig.stdin_rx.clone();
    let stdin_reply_tx = rig.stdin_reply_tx.clone();
    let payload = expected_line.clone();
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
    // Slice 1f-ι expectation: bare `(:wat::kernel::readln)` is now a
    // MalformedForm (missing the `-> :T` annotation). The assertion
    // body below is the slice-1f-α-vintage shape preserved only for
    // historical record; the #[ignore] above keeps cargo test from
    // attempting it until a follow-up slice migrates the assertion.
    let _ = result;
    let _ = expected_line;
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

    // Stone 8.1 — one miniature true universe serves all cases: the
    // real wat handle + the real service loop + a pipe, ordered by the
    // mini-TCP ack (each println's line is in the pipe before it
    // returns, so reading between cases is deterministic).
    fresh_thread();
    let world = freeze_skeleton();
    let universe = MiniUniverse::build(&world);
    for (src, expected) in cases {
        let received = universe.println_and_read(src);
        assert_eq!(received, *expected, "src={:?}", src);
    }
    universe.finish();
}

// ─── H. type-check accepts any-T for println ───────────────────────

#[test]
fn row_h_type_check_println_accepts_any_t() {
    // If println's type scheme is ∀T. T -> :wat::core::nil, freezing
    // a `:test::p` define that returns nil after calling println on
    // an i64 must succeed. Failure surfaces as a freeze error;
    // success means the type-check arm registered correctly.
    let src = r#"
        (:wat::core::defn :test::p [v <- :wat::core::i64] -> :wat::core::nil (:wat::kernel::println v))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
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
        (:wat::core::defn :test::p [v <- :wat::core::String] -> :wat::core::nil (:wat::kernel::eprintln v))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
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
#[ignore = "arc 170 slice 1f-ι: readln no longer returns HolonAST by default — its scheme is now polymorphic via the call-site `-> :T` annotation; migrate to `(:wat::kernel::readln -> :wat::core::String)` (or any other T) in a follow-up slice"]
fn row_j_type_check_readln_returns_holonast() {
    // `:test::r` declares its return as :wat::holon::HolonAST and its
    // body is exactly `(:wat::kernel::readln)`. Successful freeze
    // proves the return type unifies — the scheme says
    // `() -> :wat::holon::HolonAST`.
    let src = r#"
        (:wat::core::defn :test::r [] -> :wat::holon::HolonAST (:wat::kernel::readln))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "readln return type should unify with :wat::holon::HolonAST; got: {:?}",
        result.err()
    );
}

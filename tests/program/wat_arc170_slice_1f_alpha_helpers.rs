//! Arc 170 slice 1f-α — substrate primitives `:wat::kernel::println`,
//! `:wat::kernel::eprintln`, `:wat::kernel::readln`.
//!
//! These three primitives look up per-thread channel handles from
//! a thread-local [`wat::services::ThreadIO`] cell and run the
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
//! | C | unpopulated readln | same shape (arc 214 Stone 8.2: revived) |
//! | D | populated println sends serialized String | round-trip |
//! | E | populated eprintln sends serialized String | round-trip via stderr peer |
//! | F | populated readln returns received form | reverse direction (Stone 8.2: revived) |
//! | G | polymorphic value types — i64 / String / bool / tuple / struct | value_to_edn coverage |
//! | H | type-check accepts any-T for println | scheme registration |
//! | I | type-check accepts any-T for eprintln | scheme registration |
//! | J | type-check infers polymorphic return for readln | scheme registration (Stone 8.2: revived) |
//!
//! ThreadIO is per-thread; cargo's test-runner reuses worker
//! threads, so every populated row calls `uninstall_thread_io` on
//! exit to keep the cell clean between tests.

use std::sync::Arc;

use wat::freeze::startup_beside;
use wat::io::{PipeReader, PipeWriter, WatReader};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};
use wat::AggregateValue;
use wat::services::{
    install_thread_io, next_thread_id, spawn_service_peer, uninstall_thread_io,
    RuntimeServices, ServiceMsg, ThreadIO,
};

// ─── helpers ───────────────────────────────────────────────────────

/// Build a frozen world that contains a no-op `:user::main` plus the
/// type-check test functions for rows H, I, J. Slurped from the co-located
/// fixture `wat_arc170_slice_1f_alpha_helpers.wat` via `startup_beside`.
///
/// The invocation tests fetch a named `:probe::…` zero-arg fn from the fixture
/// and drive it via `apply_function` so the substrate's freeze pipeline runs
/// (registering the type-check arms + dispatch) without needing a meaningful
/// main body.
fn freeze_skeleton() -> wat::freeze::FrozenWorld {
    startup_beside(file!()).expect("skeleton freeze succeeds")
}

/// Stone 8.1/8.1b/8.2 — a MINIATURE TRUE UNIVERSE for all three service rows:
/// pipe-backed writers/reader (fd — cross-thread honest), the real 15-20-line
/// wat handles, the real Rust service loops, real Register exchanges. The
/// tests no longer play the service; they exercise the production pipeline
/// end to end.
struct MiniUniverse {
    sym: wat::runtime::SymbolTable,
    stdin_input_tx: wat::comms::thread::Sender<ServiceMsg<String>>,
    stdin_thread: std::thread::JoinHandle<()>,
    /// The write end of the stdin pipe. Drop this to send EOF to the stdin
    /// service loop (triggers assertion-failed! cascade in the wat handle).
    stdin_feed: PipeWriter,
    stdout_input_tx: wat::comms::thread::Sender<ServiceMsg<()>>,
    stdout_thread: std::thread::JoinHandle<()>,
    stdout_reader: PipeReader,
    stderr_input_tx: wat::comms::thread::Sender<ServiceMsg<()>>,
    stderr_thread: std::thread::JoinHandle<()>,
    stderr_reader: PipeReader,
    tid: i64,
}

impl MiniUniverse {
    fn build(world: &wat::freeze::FrozenWorld) -> Self {
        // ── stdin pipe + peer ──────────────────────────────────────────
        let (stdin_pipe_r, stdin_pipe_w) =
            wat::process::make_pipe(":test::stdin-universe").expect("pipe for the stdin reader");
        let stdin_reader = Value::io__IOReader(Arc::new(PipeReader::from_owned_fd(stdin_pipe_r)));
        let stdin_feed = PipeWriter::from_owned_fd(stdin_pipe_w);

        let stdin_handle = world
            .symbols()
            .get(":wat::kernel::services::StdInService/handle")
            .expect("/handle is in the baked stdlib")
            .clone();
        let stdin_peer = spawn_service_peer(
            "stdin",
            stdin_handle,
            stdin_reader,
            world.symbols().clone(),
            |rep: &Value| match rep {
                Value::Aggregate(sv) if sv.fields.len() >= 2 => match &sv.fields[1] {
                    Value::String(s) => Ok((**s).clone()),
                    _ => Err("StdInService Rep field[1] is not a String".into()),
                },
                _ => Err("StdInService Rep is not a Struct with ≥2 fields".into()),
            },
        );
        let wat::services::ServicePeer { input_tx: stdin_input_tx, thread: stdin_thread } = stdin_peer;

        // ── stdout pipe + peer ──────────────────────────────────────────
        let (stdout_pipe_r, stdout_pipe_w) =
            wat::process::make_pipe(":test::stdout-universe").expect("pipe for the stdout writer");
        let stdout_writer = Value::io__IOWriter(Arc::new(PipeWriter::from_owned_fd(stdout_pipe_w)));
        let stdout_reader = PipeReader::from_owned_fd(stdout_pipe_r);

        let stdout_handle = world
            .symbols()
            .get(":wat::kernel::services::StdOutService/handle")
            .expect("/handle is in the baked stdlib")
            .clone();
        let stdout_peer = spawn_service_peer(
            "stdout",
            stdout_handle,
            stdout_writer,
            world.symbols().clone(),
            |_: &Value| Ok(()),
        );
        let wat::services::ServicePeer { input_tx: stdout_input_tx, thread: stdout_thread } = stdout_peer;

        // ── stderr pipe + peer ──────────────────────────────────────────
        let (stderr_pipe_r, stderr_pipe_w) =
            wat::process::make_pipe(":test::stderr-universe").expect("pipe for the stderr writer");
        let stderr_writer = Value::io__IOWriter(Arc::new(PipeWriter::from_owned_fd(stderr_pipe_w)));
        let stderr_reader = PipeReader::from_owned_fd(stderr_pipe_r);

        let stderr_handle = world
            .symbols()
            .get(":wat::kernel::services::StdErrService/handle")
            .expect("/handle is in the baked stdlib")
            .clone();
        let stderr_peer = spawn_service_peer(
            "stderr",
            stderr_handle,
            stderr_writer,
            world.symbols().clone(),
            |_: &Value| Ok(()),
        );
        let wat::services::ServicePeer { input_tx: stderr_input_tx, thread: stderr_thread } = stderr_peer;

        // ── RS-carrying sym — all three primitives reach peers via runtime_services(). ──
        let mut sym = world.symbols().clone();
        sym.set_runtime_services(Arc::new(RuntimeServices {
            stdin_ctrl: stdin_input_tx.clone(),
            stdout_ctrl: stdout_input_tx.clone(),
            stderr_ctrl: stderr_input_tx.clone(),
        }));

        // ── Register this thread with all three peers. ───────────────────
        let tid = next_thread_id();

        let (stdin_reply_tx, stdin_reply_rx) = wat::comms::thread::pair::<Result<String, String>>();
        stdin_input_tx
            .send(ServiceMsg::Register(tid, stdin_reply_tx))
            .expect("register with the stdin service peer");

        let (stdout_reply_tx, stdout_reply_rx) = wat::comms::thread::pair::<Result<(), String>>();
        stdout_input_tx
            .send(ServiceMsg::Register(tid, stdout_reply_tx))
            .expect("register with the stdout service peer");

        let (stderr_reply_tx, stderr_reply_rx) = wat::comms::thread::pair::<Result<(), String>>();
        stderr_input_tx
            .send(ServiceMsg::Register(tid, stderr_reply_tx))
            .expect("register with the stderr service peer");

        // ── ThreadIO: all three live halves. ────────────────────────────
        install_thread_io(ThreadIO {
            stdout_reply_rx,
            stderr_reply_rx,
            thread_id: tid,
            stdin_reply_rx,
        });

        MiniUniverse {
            sym,
            stdin_input_tx,
            stdin_thread,
            stdin_feed,
            stdout_input_tx,
            stdout_thread,
            stdout_reader,
            stderr_input_tx,
            stderr_thread,
            stderr_reader,
            tid,
        }
    }

    /// Feed a line to the stdin service (writes `s + "\n"` into the pipe).
    /// The stdin handle reads this on the next Req.
    fn feed_line(&self, s: &str) {
        use wat::io::WatWriter;
        let line = format!("{}\n", s);
        self.stdin_feed
            .write_all(line.as_bytes(), wat::rust_caller_span!())
            .expect("feed_line: write");
    }

    /// Apply the named zero-arg probe fn (a readln form) and return the typed Value.
    fn readln_eval(&self, fn_name: &str) -> Result<Value, RuntimeError> {
        let func = self
            .sym
            .get(fn_name)
            .unwrap_or_else(|| panic!("no probe fn {fn_name:?} in the fixture"))
            .clone();
        apply_function(func, vec![], &self.sym, wat::rust_caller_span!())
    }

    /// Apply the named zero-arg probe fn (a println form); the mini-TCP ack means
    /// write-COMPLETED, so the line is in the pipe before this returns (ZERO-MUTEX
    /// § "use both when done matters"). Returns the written line, trimmed.
    fn println_and_read(&self, fn_name: &str) -> String {
        let func = self
            .sym
            .get(fn_name)
            .unwrap_or_else(|| panic!("no probe fn {fn_name:?} in the fixture"))
            .clone();
        let result = apply_function(func, vec![], &self.sym, wat::rust_caller_span!())
            .expect("println evals");
        assert!(matches!(result, Value::Unit), "println returns nil; fn_name={:?}", fn_name);
        self.stdout_reader
            .read_line(wat::rust_caller_span!())
            .expect("read from the stdout service's pipe")
            .expect("a written line")
            .trim()
            .to_string()
    }

    /// Apply the named zero-arg probe fn (an eprintln form) and return the line it
    /// wrote to stderr, trimmed.
    ///
    /// Arc 278 no-hidden-failures — eprintln is a TERMINATING form (a dying
    /// declaration). The mini-TCP ack means the value's EDN is in the stderr
    /// pipe BEFORE eprintln terminates; in-process the termination surfaces as
    /// an unwinding panic (`panic_any(AssertionPayload)`, the same mechanism as
    /// `assertion-failed!` / `raise!`). We catch that terminal unwind, assert it
    /// fired (eprintln must NOT return a value), then read the round-trip line
    /// the write already left in the pipe.
    fn eprintln_and_read(&self, fn_name: &str) -> String {
        let func = self
            .sym
            .get(fn_name)
            .unwrap_or_else(|| panic!("no probe fn {fn_name:?} in the fixture"))
            .clone();
        let sym = &self.sym;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            apply_function(func, vec![], sym, wat::rust_caller_span!())
        }));
        assert!(
            outcome.is_err(),
            "eprintln is terminal (arc 278): it must TERMINATE (unwind), not \
             return a value; fn_name={:?}",
            fn_name
        );
        self.stderr_reader
            .read_line(wat::rust_caller_span!())
            .expect("read from the stderr service's pipe")
            .expect("a written line — the value reached stderr before the terminal death")
            .trim()
            .to_string()
    }

    /// Teardown in the ZERO-MUTEX drop order: deregister ALL three peers,
    /// drop EVERY sender (the RS clone via sym; all originals + feed writer),
    /// THEN join all three loops — the loops exit on disconnect; joining
    /// before the drops would deadlock.
    /// Note: a panicked stdin loop (EOF cascade) joins Err — that's expected
    /// when the test explicitly drops stdin_feed before finishing.
    fn finish(self) {
        let _ = uninstall_thread_io();
        let MiniUniverse {
            sym,
            stdin_input_tx,
            stdin_thread,
            stdin_feed,
            stdout_input_tx,
            stdout_thread,
            stdout_reader,
            stderr_input_tx,
            stderr_thread,
            stderr_reader,
            tid,
        } = self;
        stdin_input_tx
            .send(ServiceMsg::Deregister(tid))
            .expect("deregister stdin");
        stdout_input_tx
            .send(ServiceMsg::Deregister(tid))
            .expect("deregister stdout");
        stderr_input_tx
            .send(ServiceMsg::Deregister(tid))
            .expect("deregister stderr");
        // Drop sym first (releases the RS clone holding all three input_tx clones).
        drop(sym);
        // Then drop the original input_tx senders (the last references;
        // disconnects the loops' input_rx so they can exit).
        drop(stdin_input_tx);
        drop(stdout_input_tx);
        drop(stderr_input_tx);
        // Drop the feed writer (EOF to the stdin pipe; but we already sent
        // Deregister above so the loop is draining normally).
        drop(stdin_feed);
        drop(stdout_reader);
        drop(stderr_reader);
        // stdin loop may have panicked (if EOF fired); log Err and continue.
        if let Err(e) = stdin_thread.join() {
            eprintln!("[test] stdin service loop panicked during finish: {:?}", e);
        }
        stdout_thread.join().expect("stdout service loop joins clean");
        stderr_thread.join().expect("stderr service loop joins clean");
    }
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
    let func = world
        .symbols()
        .get(":probe::println-42")
        .expect(":probe::println-42 is in the fixture")
        .clone();
    let err = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
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
    let func = world
        .symbols()
        .get(":probe::eprintln-42")
        .expect(":probe::eprintln-42 is in the fixture")
        .clone();
    let err = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError { kind: RuntimeErrorKind::ServiceNotRunning { op, .. }, .. } => {
            assert_eq!(op, ":wat::kernel::eprintln");
        }
        other => panic!("expected ServiceNotRunning; got {:?}", other),
    }
}

// ─── C. unpopulated readln ─────────────────────────────────────────
//
// Arc 214 Stone 8.2 — revived. The `-> :T` annotation is now required;
// unpopulated readln surfaces ServiceNotRunning (no ThreadIO installed).

#[test]
fn row_c_readln_unpopulated_returns_service_not_running() {
    fresh_thread();
    let world = freeze_skeleton();
    // :probe::readln-string's body uses the prime form directly (readln is a
    // defmacro; readln' is the kernel-restricted positional primitive that
    // eval can dispatch to).
    let func = world
        .symbols()
        .get(":probe::readln-string")
        .expect(":probe::readln-string is in the fixture")
        .clone();
    let err = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect_err("unpopulated ThreadIO must surface ServiceNotRunning");
    match err {
        RuntimeError { kind: RuntimeErrorKind::ServiceNotRunning { op, .. }, .. } => {
            assert_eq!(op, ":wat::kernel::readln'");
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
    let line = universe.println_and_read(":probe::println-42");
    assert_eq!(line, "42");
    universe.finish();
}

// ─── E. populated eprintln sends serialized String (then TERMINATES) ─
//
// Arc 214 Stone 8.1b — reborn on MiniUniverse (the real write peer
// + pipe). The legacy puppet halves are gone; the production pipeline
// runs end-to-end.
//
// Arc 278 no-hidden-failures — eprintln is now a TERMINATING form. The
// serialized String still reaches the stderr peer (the round-trip this row
// proves), but eprintln then dies; `eprintln_and_read` catches the terminal
// unwind and returns the round-trip line. The write-before-death is the
// contract this row still pins.

#[test]
fn row_e_eprintln_populated_sends_serialized_string() {
    fresh_thread();
    let world = freeze_skeleton();
    let universe = MiniUniverse::build(&world);
    let received = universe.eprintln_and_read(":probe::eprintln-hello");
    // EDN-quoted: a wat String renders as "\"hello\"".
    assert_eq!(received, "\"hello\"");
    universe.finish();
}

// ─── F. populated readln returns received form ─────────────────────
//
// Arc 214 Stone 8.2 — revived. The `-> :T` annotation is required;
// feed a raw EDN line via stdin_feed; readln parses + coerces to T.

#[test]
fn row_f_readln_populated_returns_received_form() {
    fresh_thread();
    let world = freeze_skeleton();
    let universe = MiniUniverse::build(&world);

    // Feed an EDN String value so readln can parse + coerce it.
    universe.feed_line("\"ok\"");

    let result = universe
        .readln_eval(":probe::readln-string")
        .expect("readln' succeeds");

    // The service reads the raw line, the peer routes reply_of → "ok" (without quotes),
    // then eval_kernel_readln parses the EDN "\"ok\"" → coerces to String "ok".
    match result {
        Value::String(s) => assert_eq!(s.as_str(), "ok"),
        other => panic!("expected String; got {:?}", other),
    }

    universe.finish();
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
        (":probe::println-42", "42"),
        (":probe::println-hello", "\"hello\""),
        (":probe::println-true", "true"),
        (":probe::println-false", "false"),
        // A 2-tuple — value_to_edn renders Tuples as Vectors.
        // `:wat::core::Tuple` is the verb-equals-type constructor
        // (arc 109 slice 1g). The runtime produces a Value::Tuple
        // which value_to_edn maps to an EDN Vector.
        // rune:lint(no-inlined-edn) — is the EDN tooling correct: the exact serializer output for a 2-tuple is under test, so assert_edn_eq whitespace-blindness would defeat it
        (":probe::println-tuple", "[1 2]"),
    ];

    // Stone 8.1 — one miniature true universe serves all cases: the
    // real wat handle + the real service loop + a pipe, ordered by the
    // mini-TCP ack (each println's line is in the pipe before it
    // returns, so reading between cases is deterministic).
    fresh_thread();
    let world = freeze_skeleton();
    let universe = MiniUniverse::build(&world);
    for (fn_name, expected) in cases {
        let received = universe.println_and_read(fn_name);
        assert_eq!(received, *expected, "fn_name={:?}", fn_name);
    }
    universe.finish();
}

// ─── H. type-check accepts any-T for println ───────────────────────

#[test]
fn row_h_type_check_println_accepts_any_t() {
    // If println's type scheme is ∀T. T -> :wat::core::nil, the fixture's
    // `:test::p` define (which calls println on an i64 param) must freeze
    // without error. The fixture freeze via startup_beside IS the type-check
    // assertion — a freeze failure would surface here as a panic.
    startup_beside(file!())
        .expect("println should type-check against any-T input; fixture freeze failed");
}

// ─── I. type-check accepts any-T for eprintln ──────────────────────

#[test]
fn row_i_type_check_eprintln_accepts_any_t() {
    // The fixture's `:test::p-eprintln` define calls eprintln on a String param.
    // Freeze success proves eprintln accepts any-T.
    startup_beside(file!())
        .expect("eprintln should type-check against any-T input; fixture freeze failed");
}

// ─── J. type-check accepts polymorphic return for readln ───────────
//
// Arc 214 Stone 8.2 — revived. readln's scheme is polymorphic via the
// call-site `-> :T` annotation; a defn that returns String from readln
// must freeze successfully (the `-> :wat::core::String` unifies with T).

#[test]
fn row_j_type_check_readln_returns_polymorphic_t() {
    // The fixture's `:test::r` function declares its return as :wat::core::String
    // and its body is `(:wat::kernel::readln -> :wat::core::String)`.
    // Freeze success proves the return type unifies correctly.
    startup_beside(file!())
        .expect("readln with -> :T annotation should type-check; fixture freeze failed");
}

// ─── NEW: reply-routing proof ─────────────────────────────────────
//
// Arc 214 Stone 8.2 (DESIGN-SLICE-8 named proof): two tids registered
// with the stdin peer; feed "1" then "2"; send Req(tid_a) then Req(tid_b);
// each reply_rx gets its own line — the tag routes, lines never cross.

#[test]
fn row_k_stdin_reply_routing_two_tids_never_cross() {
    fresh_thread();
    let world = freeze_skeleton();

    // ── Build stdin peer directly (no full MiniUniverse needed) ───────
    let (stdin_pipe_r, stdin_pipe_w) =
        wat::process::make_pipe(":test::stdin-routing").expect("pipe for routing proof");
    let stdin_reader = Value::io__IOReader(Arc::new(PipeReader::from_owned_fd(stdin_pipe_r)));
    let stdin_feed = PipeWriter::from_owned_fd(stdin_pipe_w);

    let stdin_handle = world
        .symbols()
        .get(":wat::kernel::services::StdInService/handle")
        .expect("/handle is in the baked stdlib")
        .clone();
    let stdin_peer = spawn_service_peer(
        "stdin",
        stdin_handle,
        stdin_reader,
        world.symbols().clone(),
        |rep: &Value| match rep {
            Value::Aggregate(sv) if sv.fields.len() >= 2 => match &sv.fields[1] {
                Value::String(s) => Ok((**s).clone()),
                _ => Err("StdInService Rep field[1] is not a String".into()),
            },
            _ => Err("StdInService Rep is not a Struct with ≥2 fields".into()),
        },
    );
    let wat::services::ServicePeer { input_tx: stdin_input_tx, thread: stdin_thread } = stdin_peer;

    // ── Register TWO tids ─────────────────────────────────────────────
    let tid_a = next_thread_id();
    let tid_b = next_thread_id();

    let (reply_tx_a, reply_rx_a) = wat::comms::thread::pair::<Result<String, String>>();
    let (reply_tx_b, reply_rx_b) = wat::comms::thread::pair::<Result<String, String>>();

    stdin_input_tx
        .send(ServiceMsg::Register(tid_a, reply_tx_a))
        .expect("register tid_a");
    stdin_input_tx
        .send(ServiceMsg::Register(tid_b, reply_tx_b))
        .expect("register tid_b");

    // ── Feed "1" then "2" into the pipe ──────────────────────────────
    {
        use wat::io::WatWriter;
        stdin_feed.write_all(b"1\n", wat::rust_caller_span!()).expect("feed 1");
        stdin_feed.write_all(b"2\n", wat::rust_caller_span!()).expect("feed 2");
    }

    // ── Send Req(tid_a) then Req(tid_b) ──────────────────────────────
    let req_a = Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::kernel::services::StdInService::Req".into(),
        vec![Value::i64(tid_a), Value::i64(524288)],
    )));
    let req_b = Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::kernel::services::StdInService::Req".into(),
        vec![Value::i64(tid_b), Value::i64(524288)],
    )));
    stdin_input_tx
        .send(ServiceMsg::Req(req_a))
        .expect("send Req(tid_a)");
    stdin_input_tx
        .send(ServiceMsg::Req(req_b))
        .expect("send Req(tid_b)");

    // ── Assert each reply_rx gets its own line (tag routes, never cross) ──
    let line_a = reply_rx_a.recv().expect("reply_rx_a receives").expect("Ok reply for tid_a");
    let line_b = reply_rx_b.recv().expect("reply_rx_b receives").expect("Ok reply for tid_b");

    // line_a should be "1" (the EDN integer 1), line_b should be "2".
    assert_eq!(line_a, "1", "tid_a's reply_rx must get line '1'");
    assert_eq!(line_b, "2", "tid_b's reply_rx must get line '2'");

    // ── Teardown ──────────────────────────────────────────────────────
    stdin_input_tx
        .send(ServiceMsg::Deregister(tid_a))
        .expect("deregister tid_a");
    stdin_input_tx
        .send(ServiceMsg::Deregister(tid_b))
        .expect("deregister tid_b");
    drop(stdin_input_tx);
    drop(stdin_feed);
    // Loop exits cleanly (Deregister sent before disconnect).
    if let Err(e) = stdin_thread.join() {
        eprintln!("[test] stdin loop panicked during routing proof teardown: {:?}", e);
    }
}

// ─── NEW: EOF cascade test ────────────────────────────────────────
//
// Arc 214 Stone 8.2 (EOF doctrine): drop the feed writer → send a Req
// → the stdin handle hits None → assertion-failed! panics the loop →
// every blocked caller's reply_rx.recv() returns Err (ChannelDisconnected).
// The loop join().is_err() IS the assertion — no catch_unwind needed.

#[test]
fn row_l_stdin_eof_cascades_to_reply_rx_disconnect() {
    fresh_thread();
    let world = freeze_skeleton();

    let (stdin_pipe_r, stdin_pipe_w) =
        wat::process::make_pipe(":test::stdin-eof").expect("pipe for eof test");
    let stdin_reader = Value::io__IOReader(Arc::new(PipeReader::from_owned_fd(stdin_pipe_r)));
    let stdin_feed = PipeWriter::from_owned_fd(stdin_pipe_w);

    let stdin_handle = world
        .symbols()
        .get(":wat::kernel::services::StdInService/handle")
        .expect("/handle is in the baked stdlib")
        .clone();
    let stdin_peer = spawn_service_peer(
        "stdin",
        stdin_handle,
        stdin_reader,
        world.symbols().clone(),
        |rep: &Value| match rep {
            Value::Aggregate(sv) if sv.fields.len() >= 2 => match &sv.fields[1] {
                Value::String(s) => Ok((**s).clone()),
                _ => Err("StdInService Rep field[1] is not a String".into()),
            },
            _ => Err("StdInService Rep is not a Struct with ≥2 fields".into()),
        },
    );
    let wat::services::ServicePeer { input_tx: stdin_input_tx, thread: stdin_thread } = stdin_peer;

    let tid = next_thread_id();
    let (reply_tx, reply_rx) = wat::comms::thread::pair::<Result<String, String>>();
    stdin_input_tx
        .send(ServiceMsg::Register(tid, reply_tx))
        .expect("register");

    // Drop the feed writer → EOF on fd 0 (from the stdin pipe reader's perspective).
    drop(stdin_feed);

    // Send a Req — the handle will read None → assertion-failed! → panic.
    let req = Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::kernel::services::StdInService::Req".into(),
        vec![Value::i64(tid), Value::i64(524288)],
    )));
    // The send may succeed (the loop hadn't panicked yet) or fail (it already did).
    let _ = stdin_input_tx.send(ServiceMsg::Req(req));

    // The caller's reply_rx.recv() returns Err (loop panicked → registry dropped).
    let got = reply_rx.recv();
    assert!(
        got.is_err(),
        "reply_rx must disconnect (Err) when the stdin loop panics on EOF; got Ok"
    );

    // The loop join().is_err() IS the EOF-cascade assertion.
    drop(stdin_input_tx);
    assert!(
        stdin_thread.join().is_err(),
        "stdin loop must join Err (panicked via assertion-failed! on EOF)"
    );
}

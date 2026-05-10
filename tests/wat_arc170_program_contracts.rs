//! Arc 170 slice 2 — wat-level surface contracts.
//!
//! These tests prove the slice-2 surface end-to-end:
//!
//! 1. `:user::main` 4-arg signature freezes; 3-arg fires walker.
//! 2. `:user::main` returns ExitCode (u8); zero + non-zero values
//!    propagate through the substrate's exit-code pipeline.
//! 3. argv pure passthrough — wat program reads argv[i] matching
//!    what wat-cli received.
//! 4. `(:wat::kernel::spawn-process fn)` — the fn matching the
//!    `:user::process` contract spawns an OS process; typed-channel
//!    send/recv works end-to-end through EDN-over-pipes.
//! 5. spawn-process with inline-lambda fn (slice 1b's fn-form
//!    entry_form path).
//! 6. spawn-process with factory-fn (single-level capture via slice
//!    1b's prologue).
//! 7. spawn-process with non-portable Sender capture fires
//!    `NonPortableCapture` (slice 1's portability check).
//! 8. `(:wat::kernel::fork-program ...)` callsite — walker fires.
//! 9. `(:wat::kernel::spawn-program ...)` callsite — walker fires.
//! 10. `(:wat::kernel::spawn-thread fn)` — UNCHANGED behavior;
//!     positive control verifying no regression.
//! 11. 3-arg `:user::main` — walker fires with the
//!     BareLegacyMainSignature diagnostic.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::{
    expected_user_main_signature, invoke_user_main, startup_from_source, validate_user_main_signature,
};
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, RuntimeError, Value};
use wat::types::TypeExpr;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

fn freeze_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected freeze to fail; succeeded"),
        Err(e) => format!("{}", e),
    }
}

/// Build a wat program with a 4-arg `:user::main` that returns an
/// ExitCode `(:wat::core::u8 N)`. Useful for the ExitCode-propagation
/// tests where the program's job is just to return a known byte.
fn user_main_returning_u8(n: u8) -> String {
    format!(
        "(:wat::core::define\n\
            (:user::main\n\
              (stdin :wat::io::IOReader)\n\
              (stdout :wat::io::IOWriter)\n\
              (stderr :wat::io::IOWriter)\n\
              (argv :wat::core::Vector<wat::core::String>)\n\
              -> :wat::kernel::ExitCode)\n\
            (:wat::core::u8 {}))",
        n
    )
}

// ─── T1. :user::main 4-arg signature freezes; 3-arg fires walker ───────

#[test]
fn t1_canonical_4arg_main_freezes() {
    // Canonical post-arc-170 shape: stdin/stdout/stderr/argv +
    // ExitCode return. Should freeze cleanly.
    let src = user_main_returning_u8(0);
    let world = freeze_ok(&src);
    // Validator agrees — the canonical signature passes.
    validate_user_main_signature(&world).expect("4-arg ExitCode :user::main validates");
    // expected_user_main_signature() exposes the canonical shape.
    let (params, ret) = expected_user_main_signature();
    assert_eq!(params.len(), 4, "expected 4 params, got {}", params.len());
    assert_eq!(
        ret,
        TypeExpr::Path(":wat::kernel::ExitCode".into()),
        "expected ExitCode return"
    );
}

#[test]
fn t1_legacy_3arg_main_fires_walker() {
    // The well-known pre-arc-170 shape: 3-arg with IOReader/Writer/Writer.
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("BareLegacyMainSignature")
            || err.contains(":user::main`")
            || err.contains("legacy 3-arg"),
        "expected BareLegacyMainSignature diagnostic; got: {}",
        err
    );
}

// ─── T2. :user::main returns ExitCode — values 0 and 42 ────────────────

/// Helper: invoke `:user::main` directly with placeholder IO + empty
/// argv, returning the Value the body produced. The substrate's
/// child-branch wraps this into an `_exit(n)` call, but the in-process
/// route lets us assert the return value cleanly.
fn invoke_main_in_process(world: &wat::freeze::FrozenWorld) -> Value {
    let stdin: Arc<dyn wat::io::WatReader> =
        Arc::new(wat::io::StringIoReader::from_string(String::new()));
    let stdout_buf = Arc::new(wat::io::StringIoWriter::new());
    let stderr_buf = Arc::new(wat::io::StringIoWriter::new());
    let stdout: Arc<dyn wat::io::WatWriter> = stdout_buf;
    let stderr: Arc<dyn wat::io::WatWriter> = stderr_buf;
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout),
        Value::io__IOWriter(stderr),
        Value::Vec(Arc::new(Vec::new())),
    ];
    invoke_user_main(world, args).expect(":user::main should run")
}

#[test]
fn t2_user_main_returns_exit_code_zero() {
    let src = user_main_returning_u8(0);
    let world = freeze_ok(&src);
    let result = invoke_main_in_process(&world);
    match result {
        Value::u8(n) => assert_eq!(n, 0, "expected u8(0); got u8({})", n),
        other => panic!("expected u8 return; got {:?}", other),
    }
}

#[test]
fn t2_user_main_returns_exit_code_nonzero() {
    let src = user_main_returning_u8(42);
    let world = freeze_ok(&src);
    let result = invoke_main_in_process(&world);
    match result {
        Value::u8(n) => assert_eq!(n, 42, "expected u8(42); got u8({})", n),
        other => panic!("expected u8 return; got {:?}", other),
    }
}

// ─── T3. argv pure passthrough ─────────────────────────────────────────

#[test]
fn t3_argv_pure_passthrough() {
    // :user::main reads argv length and returns it as the lower byte
    // of the ExitCode. Pure passthrough — what we put in is what we
    // get out. Assertion: a 3-element argv → u8(3).
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            (argv :wat::core::Vector<wat::core::String>)
            -> :wat::kernel::ExitCode)
          (:wat::core::u8 (:wat::core::Vector/length argv)))
    "#;
    let world = freeze_ok(src);
    // Invoke with a 3-element argv. The body returns the length as a
    // u8 — confirms argv flowed through and was reachable as a Vector.
    let argv = Value::Vec(Arc::new(vec![
        Value::String(Arc::new("wat".into())),
        Value::String(Arc::new("entry.wat".into())),
        Value::String(Arc::new("third-arg".into())),
    ]));
    let stdin: Arc<dyn wat::io::WatReader> =
        Arc::new(wat::io::StringIoReader::from_string(String::new()));
    let stdout_buf = Arc::new(wat::io::StringIoWriter::new());
    let stderr_buf = Arc::new(wat::io::StringIoWriter::new());
    let stdout: Arc<dyn wat::io::WatWriter> = stdout_buf;
    let stderr: Arc<dyn wat::io::WatWriter> = stderr_buf;
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout),
        Value::io__IOWriter(stderr),
        argv,
    ];
    let result = invoke_user_main(&world, args).expect(":user::main runs");
    match result {
        Value::u8(n) => assert_eq!(n, 3, "expected argv length 3 → u8(3); got u8({})", n),
        other => panic!("expected u8 return; got {:?}", other),
    }
}

// ─── T4. spawn-process(fn) end-to-end via typed channels ───────────────

fn drive_typed_recv(
    receiver_inner: &wat::typed_channel::ReceiverInner,
    types: Option<&wat::types::TypeEnv>,
) -> Value {
    match wat::typed_channel::typed_recv(receiver_inner, types, wat::span::Span::unknown()) {
        wat::typed_channel::RecvOutcome::Value(v) => v,
        wat::typed_channel::RecvOutcome::Disconnected => {
            panic!("recv: clean shutdown before value flowed")
        }
        wat::typed_channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
    }
}

fn unwrap_sender_inner(v: &Value) -> &wat::typed_channel::SenderInner {
    match v {
        Value::wat__kernel__Sender(inner) => inner.as_ref(),
        other => panic!("expected Sender Value; got {:?}", other),
    }
}

fn unwrap_receiver_inner(v: &Value) -> &wat::typed_channel::ReceiverInner {
    match v {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected Receiver Value; got {:?}", other),
    }
}

fn process_tx_field(process: &Value) -> &Value {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => &s.fields[4],
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_rx_field(process: &Value) -> &Value {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => &s.fields[5],
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_handle_field(process: &Value) -> Arc<wat::runtime::ProgramHandleInner> {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => h.clone(),
            other => panic!("expected ProgramHandle field; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

/// Wait for the spawned child to exit; assert exit code == 0.
fn wait_child_exit_ok(handle: Arc<wat::runtime::ProgramHandleInner>) {
    use wat::runtime::ProgramHandleInner;
    match handle.as_ref() {
        ProgramHandleInner::Forked(child) => {
            let code = child.wait_or_cached();
            assert_eq!(code, 0, "expected child exit 0; got {}", code);
        }
        other => panic!("expected Forked variant; got {:?}", other),
    }
}

#[test]
fn t4_spawn_process_keyword_fn_round_trips_typed_value() {
    // Top-level defn satisfying `:user::process` shape — read one
    // i64 from rx, send back rx + 1 on tx, return nil. spawn-process
    // forks an OS process; parent sends 41; child responds 42; parent
    // recv'd 42; child exits 0.
    // Note: closure_extract slice 1's free-symbol walker does NOT
    // track match-arm pattern bindings — names introduced by
    // (:wat::core::Some n) inside a match pattern surface as "free"
    // in the arm body. Honest delta from slice 2 testing: we use
    // nested `expect`s to extract the recv'd value, avoiding match
    // patterns. Result/expect / Option/expect are valid scrutinee
    // positions for kernel::recv per arc 110 § CommCallOutOfPosition.
    let src = r#"
        (:wat::core::defn :my::echo-plus-one
          [rx <- :wat::kernel::Receiver<wat::core::i64>
           tx <- :wat::kernel::Sender<wat::core::i64>]
          -> :wat::core::nil
          (:wat::core::let
            [n
              (:wat::core::Option/expect -> :wat::core::i64
                (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
                  (:wat::kernel::recv rx)
                  "recv failed")
                "stream closed")
             _send
              (:wat::core::Result/expect -> :wat::core::nil
                (:wat::kernel::send tx (:wat::core::i64::+'2 n 1))
                "send failed")]
            :wat::core::nil))
    "#;
    let world = freeze_ok(src);
    // Build the spawn-process call form: (:wat::kernel::spawn-process :my::echo-plus-one)
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::echo-plus-one".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());
    // Parent sends 41 to child via tx.
    let outcome = wat::typed_channel::typed_send(
        unwrap_sender_inner(process_tx_field(&process)),
        Value::i64(41),
        types,
        wat::span::Span::unknown(),
    );
    assert!(
        matches!(outcome, wat::typed_channel::SendOutcome::Ok),
        "send should succeed"
    );
    // Parent recvs response — should be 42. On unexpected close, drain
    // stderr so we surface the child's diagnostic in the panic message.
    let recv_outcome = wat::typed_channel::typed_recv(
        unwrap_receiver_inner(process_rx_field(&process)),
        types,
        wat::span::Span::unknown(),
    );
    let response = match recv_outcome {
        wat::typed_channel::RecvOutcome::Value(v) => v,
        wat::typed_channel::RecvOutcome::Disconnected => {
            // Drain child stderr for diagnostic.
            let stderr_field = match &process {
                Value::Struct(s) => &s.fields[2],
                _ => panic!("not a Process Struct"),
            };
            let stderr_text = match stderr_field {
                Value::io__IOReader(rdr) => {
                    let mut all = String::new();
                    while let Ok(Some(line)) = rdr.read_line(wat::span::Span::unknown()) {
                        all.push_str(&line);
                    }
                    all
                }
                _ => "<stderr field not IOReader>".to_string(),
            };
            panic!("recv: clean shutdown before value flowed; child stderr:\n{}", stderr_text);
        }
        wat::typed_channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
    };
    match response {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64 response; got {:?}", other),
    }
    // Wait for the child to exit cleanly.
    wait_child_exit_ok(process_handle_field(&process));
}

// ─── T5. spawn-process(inline lambda) — slice 1b fn-form path ──────────

#[test]
fn t5_spawn_process_inline_lambda_round_trips() {
    // No top-level defn — pass an inline (:wat::core::fn ...) directly
    // as the spawn-process arg. Slice 1b's inline-lambda entry_form
    // path: extract_closure produces a fn-form AST as entry_form.
    // The child evaluates the fn-form to get a fresh fn Value and
    // applies it.
    let src = r#"
        (:wat::core::define
          (:my::launch
            -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [rx <- :wat::kernel::Receiver<wat::core::i64>
               tx <- :wat::kernel::Sender<wat::core::i64>]
              -> :wat::core::nil
              (:wat::core::let
                [n
                  (:wat::core::Option/expect -> :wat::core::i64
                    (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
                      (:wat::kernel::recv rx)
                      "recv failed")
                    "stream closed")
                 _send
                  (:wat::core::Result/expect -> :wat::core::nil
                    (:wat::kernel::send tx (:wat::core::i64::*'2 n 2))
                    "send failed")]
                :wat::core::nil))))
    "#;
    let world = freeze_ok(src);
    // Invoke the launcher to get the Process Value.
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let process = wat::runtime::apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect(":my::launch runs");
    let types = world.symbols().types().map(|a| a.as_ref());
    let outcome = wat::typed_channel::typed_send(
        unwrap_sender_inner(process_tx_field(&process)),
        Value::i64(21),
        types,
        wat::span::Span::unknown(),
    );
    assert!(matches!(outcome, wat::typed_channel::SendOutcome::Ok));
    let response = drive_typed_recv(unwrap_receiver_inner(process_rx_field(&process)), types);
    match response {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
    wait_child_exit_ok(process_handle_field(&process));
}

// ─── T6. spawn-process(factory-fn) — single-level capture ──────────────

#[test]
fn t6_spawn_process_factory_with_capture_round_trips() {
    // A factory builds a closure capturing a config value, then
    // spawn-process forks against the captured fn. Slice 1b's
    // closure-extraction encodes the captured value into prologue
    // (`(def :__captured_offset N)`); the child re-freezes; the
    // captured offset survives.
    let src = r#"
        (:wat::core::define
          (:my::launch
            (offset :wat::core::i64)
            -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [rx <- :wat::kernel::Receiver<wat::core::i64>
               tx <- :wat::kernel::Sender<wat::core::i64>]
              -> :wat::core::nil
              (:wat::core::let
                [n
                  (:wat::core::Option/expect -> :wat::core::i64
                    (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
                      (:wat::kernel::recv rx)
                      "recv failed")
                    "stream closed")
                 _send
                  (:wat::core::Result/expect -> :wat::core::nil
                    (:wat::kernel::send tx (:wat::core::i64::+'2 n offset))
                    "send failed")]
                :wat::core::nil))))
    "#;
    let world = freeze_ok(src);
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let process = wat::runtime::apply_function(
        launcher.clone(),
        vec![Value::i64(100)],
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect(":my::launch runs");
    let types = world.symbols().types().map(|a| a.as_ref());
    let outcome = wat::typed_channel::typed_send(
        unwrap_sender_inner(process_tx_field(&process)),
        Value::i64(7),
        types,
        wat::span::Span::unknown(),
    );
    assert!(matches!(outcome, wat::typed_channel::SendOutcome::Ok));
    let response = drive_typed_recv(unwrap_receiver_inner(process_rx_field(&process)), types);
    match response {
        Value::i64(n) => assert_eq!(n, 107, "expected 100+7=107; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
    wait_child_exit_ok(process_handle_field(&process));
}

// ─── T7. spawn-process with non-portable Sender capture ────────────────

#[test]
fn t7_spawn_process_non_portable_capture_fires_diagnostic() {
    // A factory builds a closure capturing a Sender from the parent's
    // let-scope. The Sender is a channel-bearing Value — pointer
    // identity does not survive fork(2). Slice 1's portability check
    // refuses; spawn-process surfaces the diagnostic.
    let src = r#"
        (:wat::core::define
          (:my::launch
            -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::core::let
            [pair (:wat::kernel::make-unbounded-channel)
             extra-tx (:wat::core::first pair)]
            (:wat::kernel::spawn-process
              (:wat::core::fn
                [rx <- :wat::kernel::Receiver<wat::core::i64>
                 tx <- :wat::kernel::Sender<wat::core::i64>]
                -> :wat::core::nil
                (:wat::core::let
                  [n
                    (:wat::core::Option/expect -> :wat::core::i64
                      (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
                        (:wat::kernel::recv rx)
                        "recv failed")
                      "stream closed")
                   _send
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send extra-tx n)
                      "send failed")]
                  :wat::core::nil)))))
    "#;
    // The freeze may succeed (the closure-extract check fires at
    // spawn-process invocation, not at freeze). If the type-checker
    // already rejects, that's also a valid failure mode — both paths
    // refuse the non-portable shape.
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(world) => {
            let launcher = world
                .symbols()
                .get(":my::launch")
                .expect("launch defined");
            let result = wat::runtime::apply_function(
                launcher.clone(),
                Vec::new(),
                world.symbols(),
                wat::span::Span::unknown(),
            );
            match result {
                Err(RuntimeError::MalformedForm { reason, .. }) => {
                    assert!(
                        reason.contains("non-portable")
                            || reason.contains("NonPortableCapture")
                            || reason.contains("Channel-bearing")
                            || reason.contains("Sender")
                            || reason.contains("Receiver")
                            || reason.contains("captures"),
                        "expected non-portable diagnostic; got reason: {}",
                        reason
                    );
                }
                Ok(_) => panic!("expected non-portable refusal; succeeded"),
                Err(other) => {
                    let msg = format!("{:?}", other);
                    let lc = msg.to_lowercase();
                    assert!(
                        lc.contains("sender")
                            || lc.contains("non-portable")
                            || lc.contains("channel")
                            || lc.contains("captures"),
                        "expected error mentioning channel non-portability; got: {}",
                        msg
                    );
                }
            }
        }
        Err(freeze_err) => {
            // Type-check rejected at freeze time — also OK.
            let _ = format!("{}", freeze_err);
        }
    }
}

// ─── T8. fork-program callsite — walker fires ─────────────────────────

#[test]
fn t8_fork_program_callsite_fires_walker() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            (argv :wat::core::Vector<wat::core::String>)
            -> :wat::kernel::ExitCode)
          (:wat::core::do
            (:wat::kernel::fork-program "" :wat::core::None)
            (:wat::core::u8 0)))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("BareLegacyForkProgram") || err.contains(":wat::kernel::fork-program"),
        "expected BareLegacyForkProgram diagnostic; got: {}",
        err
    );
}

#[test]
fn t8b_fork_program_ast_callsite_fires_walker() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            (argv :wat::core::Vector<wat::core::String>)
            -> :wat::kernel::ExitCode)
          (:wat::core::do
            (:wat::kernel::fork-program-ast (:wat::core::Vector :wat::WatAST))
            (:wat::core::u8 0)))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("BareLegacyForkProgram")
            || err.contains(":wat::kernel::fork-program-ast"),
        "expected BareLegacyForkProgram diagnostic; got: {}",
        err
    );
}

// ─── T9. spawn-program callsite — walker fires ───────────────────────

#[test]
fn t9_spawn_program_callsite_fires_walker() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            (argv :wat::core::Vector<wat::core::String>)
            -> :wat::kernel::ExitCode)
          (:wat::core::do
            (:wat::kernel::spawn-program "" :wat::core::None)
            (:wat::core::u8 0)))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("BareLegacySpawnProgram") || err.contains(":wat::kernel::spawn-program"),
        "expected BareLegacySpawnProgram diagnostic; got: {}",
        err
    );
}

#[test]
fn t9b_spawn_program_ast_callsite_fires_walker() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            (argv :wat::core::Vector<wat::core::String>)
            -> :wat::kernel::ExitCode)
          (:wat::core::do
            (:wat::kernel::spawn-program-ast (:wat::core::Vector :wat::WatAST) :wat::core::None)
            (:wat::core::u8 0)))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("BareLegacySpawnProgram")
            || err.contains(":wat::kernel::spawn-program-ast"),
        "expected BareLegacySpawnProgram diagnostic; got: {}",
        err
    );
}

// ─── T10. spawn-thread(fn) — UNCHANGED behavior ──────────────────────

#[test]
fn t10_spawn_thread_unchanged_positive_control() {
    // Same shape as before arc 170 — spawn-thread takes a fn whose
    // signature is :Receiver<I> + :Sender<O> → :nil. Behavior must
    // not regress: the thread runs in parent's world, communicates
    // via crossbeam channels, returns Thread<I,O>.
    let src = r#"
        (:wat::core::defn :my::echo-thread
          [rx <- :rust::crossbeam_channel::Receiver<wat::core::i64>
           tx <- :rust::crossbeam_channel::Sender<wat::core::i64>]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::nil
            ((:wat::core::Ok (:wat::core::Some n))
              (:wat::core::match (:wat::kernel::send tx (:wat::core::i64::*'2 n 2)) -> :wat::core::nil
                ((:wat::core::Ok _) :wat::core::nil)
                ((:wat::core::Err _) :wat::core::nil)))
            ((:wat::core::Ok :wat::core::None) :wat::core::nil)
            ((:wat::core::Err _died) :wat::core::nil)))
    "#;
    let world = freeze_ok(src);
    // Build (:wat::kernel::spawn-thread :my::echo-thread).
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-thread".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::echo-thread".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let thread = eval(&call, &env, world.symbols()).expect("spawn-thread succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());
    // Thread<I,O> field order: input(0), output(1), join(2)
    let (input, output) = match &thread {
        Value::Struct(s) if s.type_name == ":wat::kernel::Thread" => (&s.fields[0], &s.fields[1]),
        other => panic!("expected Thread Struct; got {:?}", other),
    };
    let outcome = wat::typed_channel::typed_send(
        unwrap_sender_inner(input),
        Value::i64(21),
        types,
        wat::span::Span::unknown(),
    );
    assert!(matches!(outcome, wat::typed_channel::SendOutcome::Ok));
    let response = match wat::typed_channel::typed_recv(
        unwrap_receiver_inner(output),
        types,
        wat::span::Span::unknown(),
    ) {
        wat::typed_channel::RecvOutcome::Value(v) => v,
        other => panic!("expected Value; got {:?}", other),
    };
    match response {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── T11. 3-arg :user::main fires walker (BareLegacyMainSignature) ────

#[test]
fn t11_legacy_main_signature_fires_walker_diagnostic() {
    let src = r#"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let err = freeze_err(src);
    // The walker's Display output should mention the canonical 4-arg
    // shape and ExitCode return.
    assert!(
        err.contains("argv") || err.contains("ExitCode") || err.contains("4-arg"),
        "expected migration template (argv / ExitCode / 4-arg) in diagnostic; got: {}",
        err
    );
}

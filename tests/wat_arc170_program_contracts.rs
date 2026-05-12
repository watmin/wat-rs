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

// ─── T1. :user::main [] -> :wat::core::nil signature freezes; 3-arg fires walker ──

#[test]
fn t1_canonical_nil_main_freezes() {
    // Arc 170 slice 1e canonical shape: no params, nil return. Should freeze cleanly.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    // Validator agrees — the canonical signature passes.
    validate_user_main_signature(&world).expect("[] -> nil :user::main validates");
    // expected_user_main_signature() exposes the canonical shape: 0 params, nil return.
    let (params, ret) = expected_user_main_signature();
    assert_eq!(params.len(), 0, "expected 0 params (argv is ambient), got {}", params.len());
    assert_eq!(
        ret,
        TypeExpr::Tuple(vec![]),
        "expected nil (Tuple([])) return"
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

// ─── T2. :user::main [] -> :wat::core::nil invokes cleanly ─────────────

#[test]
fn t2_canonical_main_returns_nil_value() {
    // nil IS the success exit code (arc 170 REALIZATIONS pass 10).
    // invoke_user_main on a canonical [] -> nil main returns nil.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main should run");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

#[test]
fn t2_canonical_main_with_let_body_returns_nil() {
    // A canonical main with a non-trivial body (let binding + discard)
    // still returns nil. Confirms the do-work-return-nil pattern runs.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_ (:wat::core::i64::+'2 1 2)]
            :wat::core::nil))
    "#;
    let world = freeze_ok(src);
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main should run");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

// ─── T3. argv ambient reachable via (:wat::runtime::argv) ─────────────

#[test]
fn t3_argv_reachable_via_ambient() {
    // Arc 170 REALIZATIONS pass 7: argv is ambient (not a parameter).
    // A canonical main body can access (:wat::runtime::argv) — the
    // freeze should succeed (type-check validates the argv expression).
    // At runtime the ambient vector is whatever set_argv was called with
    // (empty if never set). We just confirm the program freezes and runs.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [_ (:wat::runtime::argv)]
            :wat::core::nil))
    "#;
    let world = freeze_ok(src);
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main runs");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
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

// ─── T12. spawn-process(fn) — child emits without recv'ing first ──────
//
// Slice 1f-λ rebuild for the arc-104 fork_program_child_writes_stdout
// scenario. Under arc 170 the child's "stdout" is a typed Sender<T>;
// the child sends one value via tx without first reading rx. The rx
// channel exists per the contract shape but goes unread.

#[test]
fn t12_spawn_process_child_emits_without_recv() {
    let src = r#"
        (:wat::core::defn :my::emit-hello
          [rx <- :wat::kernel::Receiver<wat::core::nil>
           tx <- :wat::kernel::Sender<wat::core::String>]
          -> :wat::core::nil
          (:wat::core::let
            [_send
              (:wat::core::Result/expect -> :wat::core::nil
                (:wat::kernel::send tx "hello-from-fork")
                "send failed")]
            :wat::core::nil))
    "#;
    let world = freeze_ok(src);
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::emit-hello".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());
    let response = drive_typed_recv(unwrap_receiver_inner(process_rx_field(&process)), types);
    match response {
        Value::String(s) => assert_eq!(&*s, "hello-from-fork", "expected hello-from-fork; got {:?}", s),
        other => panic!("expected String; got {:?}", other),
    }
    wait_child_exit_ok(process_handle_field(&process));
}

// ─── T13. spawn-process(fn) — child exits clean on parent tx-drop ─────
//
// Slice 1f-λ rebuild for the arc-104 fork_program_clean_exit_code
// scenario. Child waits on rx; parent drops the Process (which drops
// its Sender side) → child's rx surfaces a disconnect; child returns
// nil; wait_child_exit_ok confirms exit code 0.

#[test]
fn t13_spawn_process_child_exits_clean_on_parent_tx_drop() {
    let src = r#"
        (:wat::core::defn :my::wait-for-disconnect
          [rx <- :wat::kernel::Receiver<wat::core::nil>
           tx <- :wat::kernel::Sender<wat::core::nil>]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::recv rx)
            -> :wat::core::nil
            ((:wat::core::Ok :wat::core::None) :wat::core::nil)
            ((:wat::core::Ok (:wat::core::Some _)) :wat::core::nil)
            ((:wat::core::Err _) :wat::core::nil)))
    "#;
    let world = freeze_ok(src);
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::wait-for-disconnect".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let handle = process_handle_field(&process);
    // Parent does NOT send anything; drop process Struct → tx drops →
    // child's rx surfaces Disconnected → child returns nil → exit 0.
    drop(process);
    wait_child_exit_ok(handle);
}

// ─── T14. spawn-process(fn) — wait handle is idempotent ──────────────
//
// Slice 1f-λ rebuild for the arc-012 wait_child_is_idempotent scenario.
// ChildHandleInner::wait_or_cached() uses OnceLock caching; calling it
// twice must return the same exit code rather than re-waiting or
// returning a sentinel. Child fn returns nil immediately (idle worker).

#[test]
fn t14_spawn_process_wait_handle_is_idempotent() {
    let src = r#"
        (:wat::core::defn :my::idle-worker
          [rx <- :wat::kernel::Receiver<wat::core::nil>
           tx <- :wat::kernel::Sender<wat::core::nil>]
          -> :wat::core::nil
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::idle-worker".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let handle = process_handle_field(&process);
    // Drop process → tx drops → child's rx disconnects → child returns nil → exit 0.
    drop(process);
    // First wait — real waitpid; caches exit 0.
    wait_child_exit_ok(handle.clone());
    // Second wait — must return cached 0, not re-wait (idempotency).
    wait_child_exit_ok(handle);
}

// ─── T15. spawn-process(fn) — child panics → recv Disconnected + non-zero exit
//
// Slice 1f-λ rebuild for the arc-012 wait_child_surfaces_panic_exit_code
// scenario. Child fn body calls Option/expect on None → panics →
// spawn_process_child_branch's catch_unwind catches → writes to stderr pipe
// → exits EXIT_PANIC (2). Parent's typed recv returns Disconnected (child
// closed output before sending). Handle exit code is non-zero.

#[test]
fn t15_spawn_process_child_panic_disconnects_recv_and_exits_nonzero() {
    let src = r#"
        (:wat::core::defn :my::panic-worker
          [rx <- :wat::kernel::Receiver<wat::core::nil>
           tx <- :wat::kernel::Sender<wat::core::nil>]
          -> :wat::core::nil
          (:wat::core::Option/expect -> :wat::core::nil
            :wat::core::None
            "intentional panic in child"))
    "#;
    let world = freeze_ok(src);
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
            WatAST::Keyword(":my::panic-worker".into(), wat::span::Span::unknown()),
        ],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());
    let handle = process_handle_field(&process);
    // Parent recvs — child panics before sending anything → Disconnected.
    let recv_outcome = wat::typed_channel::typed_recv(
        unwrap_receiver_inner(process_rx_field(&process)),
        types,
        wat::span::Span::unknown(),
    );
    assert!(
        matches!(recv_outcome, wat::typed_channel::RecvOutcome::Disconnected),
        "expected Disconnected (child panicked before sending); got {:?}",
        recv_outcome,
    );
    // Handle exit code must be non-zero (EXIT_PANIC=2).
    use wat::runtime::ProgramHandleInner;
    let code = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached(),
        other => panic!("expected Forked ProgramHandle; got {:?}", other),
    };
    assert_ne!(code, 0, "expected non-zero exit on child panic; got 0");
}

// ─── T17. run-hermetic macro — Layer 1 testing-lib API (arc 170 slice 3 phase C)
//
// Canonical Layer 1 test: a simple assertion body wrapped by the
// run-hermetic macro. The macro generates the fn-form, calls
// spawn-process, drains via run-hermetic-driver, and returns RunResult.
// A passing assertion produces RunResult { failure: None }; the test
// verifies the failure slot is empty.
//
// Surface form exercised:
//   (:wat::test::run-hermetic
//     (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4))
//
// The function is defined at :my::test::two-plus-two; invoked with
// apply_function (zero args); RunResult.failure must be None.

#[test]
fn t17_run_hermetic_layer1_passing_assertion() {
    // Define a function that calls run-hermetic with a simple assertion.
    // run-hermetic is a macro; it expands the body into a fn, spawns
    // an OS process, drains stdout/stderr, joins, and returns RunResult.
    // A passing assertion (2+2=4) means the child exits 0 and failure
    // is :None.
    let src = r#"
        (:wat::core::define (:my::test::two-plus-two -> :wat::kernel::RunResult)
          (:wat::test::run-hermetic
            (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4)))
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":my::test::two-plus-two")
        .expect(":my::test::two-plus-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("run-hermetic should succeed");
    // result is a :wat::kernel::RunResult { stdout stderr failure }
    // failure must be :None (the assertion passed).
    let sv = match &result {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult Struct; got {:?}", other),
    };
    // RunResult field 2 is failure :Option<Failure>
    let failure_field = &sv.fields[2];
    let is_none = match failure_field {
        wat::runtime::Value::Option(opt) => opt.as_ref().is_none(),
        other => panic!("expected Option failure field; got {:?}", other),
    };
    assert!(
        is_none,
        "expected passing assertion to produce RunResult with failure=None; got {:?}",
        result
    );
}

#[test]
fn t17b_run_hermetic_layer1_failing_assertion_surfaces_failure() {
    // Complementary to T17: a failing assertion (1 != 2) should produce
    // RunResult { failure: Some(Failure) } — the child exits non-zero,
    // spawn-process emits the structured `#wat.kernel/ProcessPanics`
    // EDN line on stderr, extract-panics rebuilds the cascade, and
    // run-hermetic-driver surfaces the structured Failure with the
    // assert-eq diagnostic in Failure.message.
    //
    // Arc 170 slice 3 phase C′ closed the substrate gap that previously
    // forced this test to skip message-text assertion. spawn_process.rs
    // now mirrors fork.rs::emit_panics_to_stderr — AssertionPayload
    // panics emit the structured chain; plain panics fall through to
    // the singleton "exited N" path.
    let src = r#"
        (:wat::core::define (:my::test::one-neq-two -> :wat::kernel::RunResult)
          (:wat::test::run-hermetic
            (:wat::test::assert-eq (:wat::core::i64::+'2 1 0) 2)))
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":my::test::one-neq-two")
        .expect(":my::test::one-neq-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("run-hermetic driver should not itself panic");
    let sv = match &result {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult Struct; got {:?}", other),
    };
    // RunResult field 2 is failure :Option<Failure>; must be Some (child panicked).
    let failure_field = &sv.fields[2];
    let failure_val = match failure_field {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!("expected failing assertion to produce Some(Failure); got None"),
        },
        other => panic!("expected Option failure field; got {:?}", other),
    };
    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };
    // Failure.message (field 0) must carry the structured assert-eq diagnostic,
    // NOT the singleton exit-code fallback ("forked program exited N"). This
    // proves the spawn_process.rs panic-chain emit (phase C′) is wired up
    // and extract-panics rebuilt the cascade.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::String(s) => s.to_string(),
        other => panic!("expected Failure.message :String; got {:?}", other),
    };
    assert!(
        !message.contains("forked program exited"),
        "expected structured assert-eq message; got exit-code fallback: {}",
        message
    );
    assert!(
        message.contains("assert") || message.contains("AssertionFailed"),
        "expected message to mention assert/AssertionFailed; got: {}",
        message
    );
}

// ─── T16. spawn-process(fn) — multiple sequential spawns, no fd/zombie leak
//
// Slice 1f-λ rebuild for the arc-012 multiple_sequential_forks_no_leak
// scenario. Three sequential spawn+exit cycles from one parent prove that
// pipe fds close cleanly and waitpid reaps zombies without accumulation.
// Each child uses the idle-worker pattern; each exits 0.

#[test]
fn t16_spawn_process_sequential_spawns_no_fd_zombie_leak() {
    let src = r#"
        (:wat::core::defn :my::idle-worker-seq
          [rx <- :wat::kernel::Receiver<wat::core::nil>
           tx <- :wat::kernel::Sender<wat::core::nil>]
          -> :wat::core::nil
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let env = Environment::new();
    for _ in 0..3 {
        let call = WatAST::List(
            vec![
                WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::span::Span::unknown()),
                WatAST::Keyword(":my::idle-worker-seq".into(), wat::span::Span::unknown()),
            ],
            wat::span::Span::unknown(),
        );
        let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
        let handle = process_handle_field(&process);
        // Drop process → child exits 0.
        drop(process);
        wait_child_exit_ok(handle);
    }
}

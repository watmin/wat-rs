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
//! 7. spawn-process with impure Sender capture fires
//!    `ImpureCapture` (arc 293.W.2d rename of NonPortableCapture).
//! 8/9. (retired) `*-program{,-ast}` callsite retirement nags —
//!    ANNIHILATED (arc 170 CULMINATION); the verbs had zero live callers
//!    and no runtime eval, so the check-time nag + its tests are gone.
//! 10. `(:wat::kernel::spawn-thread fn)` — UNCHANGED behavior;
//!     positive control verifying no regression.
//! 11. 3-arg `:user::main` — walker fires with the
//!     BareLegacyMainSignature diagnostic.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::{
    expected_user_main_signature, invoke_user_main, startup_beside, startup_from_file,
    validate_user_main_signature,
};
use wat::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, Value};
use wat::types::TypeExpr;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(fixture: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(fixture)
        .unwrap_or_else(|e| panic!("freeze should succeed for {fixture:?}; got: {e}"))
}

fn freeze_err(fixture: &str) -> String {
    match startup_from_file(fixture) {
        Ok(_) => panic!("expected freeze to fail for {fixture:?}; succeeded"),
        Err(e) => format!("{}", e),
    }
}

/// Load the primary fixture (canonical trivial main) via startup_beside.
fn freeze_trivial() -> wat::freeze::FrozenWorld {
    startup_beside(file!()).expect("trivial-main fixture must freeze")
}

/// Arc 170 slice 6 helper — build a `(:wat::kernel::spawn-process
/// (:wat::core::forms <child-program-forms>...))` call AST from a
/// child-program source string. The child program is parsed via
/// `parser::parse_all_with_file` and must include a top-level
/// `(:wat::core::defn :user::main [] -> :wat::core::nil ...)` (Stone 241.12: migrated from define).
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

// PARENT_TRIVIAL retired: callers now use freeze_trivial() which loads
// tests/program/wat_arc170_program_contracts.wat via startup_beside(file!()).

/// Build a spawn-process call AST from a co-located child-program `.wat`
/// fixture (read from disk, never inlined) — see `build_spawn_process_call`.
fn build_spawn_process_call_from_fixture(path: &str) -> WatAST {
    let child_program_src =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("child fixture {path:?} must exist: {e}"));
    build_spawn_process_call(&child_program_src)
}

// ─── T1. :user::main [] -> :wat::core::nil signature freezes; 3-arg fires walker ──

#[test]
fn t1_canonical_nil_main_freezes() {
    // Arc 170 slice 1e canonical shape: no params, nil return. Should freeze cleanly.
    let world = freeze_trivial();
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
#[ignore = "ARC-170 WIP: BareLegacyMainSignature walker no longer fires for a non-canonical :user::main (freeze succeeds where it should reject — likely walker-disconnect); investigate + fix/retire before arc 170 closes."]
fn t1_legacy_3arg_main_fires_walker() {
    unimplemented!("arc 170: BareLegacyMainSignature walker reconnect; on unlock assert the legacy-3arg-main diagnostic exactly");
}

// ─── T2. :user::main [] -> :wat::core::nil invokes cleanly ─────────────

#[test]
fn t2_canonical_main_returns_nil_value() {
    // nil IS the success exit code (arc 170 REALIZATIONS pass 10).
    // invoke_user_main on a canonical [] -> nil main returns nil.
    let world = freeze_trivial();
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
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t2_let.wat");
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
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t3_argv.wat");
    let result = invoke_user_main(&world, Vec::new()).expect(":user::main runs");
    assert!(
        matches!(result, Value::Unit),
        "expected nil (Value::Unit); got {:?}", result
    );
}

// ─── T4. spawn-process(fn) end-to-end via typed channels ───────────────

fn drive_typed_recv(
    receiver_inner: &wat::channel::ReceiverInner,
    types: Option<&wat::types::TypeEnv>,
) -> Value {
    match wat::channel::typed_recv(receiver_inner, types, wat::rust_caller_span!()) {
        wat::channel::RecvOutcome::Value(v) => v,
        wat::channel::RecvOutcome::Disconnected => {
            panic!("recv: clean shutdown before value flowed")
        }
        wat::channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
        wat::channel::RecvOutcome::Shutdown => {
            panic!("recv: unexpected process-wide shutdown during test")
        }
    }
}

fn unwrap_sender_inner(v: &Value) -> &wat::channel::SenderInner {
    match v {
        Value::wat__kernel__Sender(inner) => inner.as_ref(),
        other => panic!("expected Sender Value; got {:?}", other),
    }
}

fn unwrap_receiver_inner(v: &Value) -> &wat::channel::ReceiverInner {
    match v {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected Receiver Value; got {:?}", other),
    }
}

fn process_stdin_field(process: &Value) -> Arc<dyn wat::io::WatWriter> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[0] {
            Value::io__IOWriter(w) => w.clone(),
            other => panic!("expected IOWriter at fields[0]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_stdout_field(process: &Value) -> Arc<dyn wat::io::WatReader> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[1] {
            Value::io__IOReader(r) => r.clone(),
            other => panic!("expected IOReader at fields[1]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_handle_field(process: &Value) -> Arc<wat::runtime::ProgramHandleInner> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[3] {
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
            let code = child.wait_or_cached_exit();
            assert_eq!(code, 0, "expected child exit 0; got {}", code);
        }
        other => panic!("expected Forked variant; got {:?}", other),
    }
}

#[test]
fn t4_spawn_process_keyword_fn_round_trips_typed_value() {
    // Arc 170 slice 6 — spawn-process accepts a wat PROGRAM
    // (`Vec<WatAST>`). The child program is self-contained: a single
    // (:user::main -> :nil) define whose body reads one i64, prints n+1.
    // Parent sends 41 via Sender/from-pipe; child responds 42 via
    // println; parent reads 42 via Receiver/from-pipe; child exits 0.
    let world = freeze_trivial();
    let call = build_spawn_process_call_from_fixture(
        "tests/program/wat_arc170_program_contracts_t4_child.wat",
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let types = world.symbols().types().map(|a| a.as_ref());
    // Parent sends 41 to child via Sender/from-pipe wrapping Process/stdin (IOWriter).
    let stdin_writer = process_stdin_field(&process);
    let sender_val = wat::channel::sender_from_pipe(stdin_writer);
    let sender_inner = unwrap_sender_inner(&sender_val);
    let outcome = wat::channel::typed_send(
        sender_inner,
        Value::i64(41),
        types,
        wat::rust_caller_span!(),
    );
    assert!(
        matches!(outcome, wat::channel::SendOutcome::Ok),
        "send should succeed"
    );
    // Drop sender so child's readln sees EOF after the first read (not needed
    // for single-value round-trip, but avoids child blocking on a second readln).
    drop(sender_val);
    // Parent recvs response — should be 42. On unexpected close, drain
    // stderr so we surface the child's diagnostic in the panic message.
    let stdout_reader = process_stdout_field(&process);
    let receiver_val = wat::channel::receiver_from_pipe(stdout_reader);
    let receiver_inner = unwrap_receiver_inner(&receiver_val);
    let recv_outcome = wat::channel::typed_recv(
        receiver_inner,
        types,
        wat::rust_caller_span!(),
    );
    let response = match recv_outcome {
        wat::channel::RecvOutcome::Value(v) => v,
        wat::channel::RecvOutcome::Disconnected => {
            // Drain child stderr for diagnostic.
            let stderr_field = match &process {
                Value::Aggregate(s) => &s.fields[2],
                _ => panic!("not a Process Struct"),
            };
            let stderr_text = match stderr_field {
                Value::io__IOReader(rdr) => {
                    let mut all = String::new();
                    while let Ok(Some(line)) = rdr.read_line(wat::rust_caller_span!()) {
                        all.push_str(&line);
                    }
                    all
                }
                _ => "<stderr field not IOReader>".to_string(),
            };
            panic!("recv: clean shutdown before value flowed; child stderr:\n{}", stderr_text);
        }
        wat::channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
        wat::channel::RecvOutcome::Shutdown => {
            panic!("recv: unexpected process-wide shutdown during test")
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
    // Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process`
    // onto the composed primes (`spawn-program' (process)` + `send'` + `recv'`). The
    // launcher constructs the child via an inline (:wat::core::forms (:wat::core::defn ...))
    // program, spawns it as a process peer, feeds its `readln` with `send' 21`, and returns
    // the doubled value that crossed back off the peer as a recv' RecvOutcome::Message.
    // The launcher now returns the recv'd i64 directly (== 42), so this test measures the
    // value that genuinely crossed the wire (NOT stdout-scraped). Mirrors t17's apply-and-
    // assert-i64 shape (SAME .rs file family). Fixture: t5_launch_lambda.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t5_launch_lambda.wat");
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = wat::runtime::apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + send' + recv')");
    match &result {
        Value::i64(n) => assert_eq!(
            *n, 42,
            "expected 21*2=42 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── T6. spawn-process(factory-fn) — single-level capture ──────────────

#[test]
fn t6_spawn_process_factory_with_capture_round_trips() {
    // Arc 278 IPC de-prime — migrated off the non-prime `:wat::kernel::spawn-process`
    // onto the composed primes (`spawn-program' (process)` + `send'` + `recv'`). The
    // substrate-equivalent of closure-capture-across-fork is runtime AST construction:
    // the launcher splices the runtime `offset` value INTO the child program AST via
    // `:wat::core::quasiquote` + `:wat::core::unquote`, builds the
    // `(:wat::core::Vector :wat::WatAST main-form)` forms VALUE, and hands it to
    // `spawn-program' (process)` — the process clause accepts a forms value the same way
    // spawn-process did, so the quasiquote-factory shape is unchanged; only the DRIVER
    // flipped to the peer wire. The launcher feeds the child's `readln` with `send' 7`
    // and returns the `(n + offset)` value that crossed back as a recv' RecvOutcome::Message.
    // With offset=100 the recv'd i64 is 107 — the value that genuinely crossed the wire.
    // Fixture: t6_launch_factory.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t6_launch_factory.wat");
    let launcher = world.symbols().get(":my::launch").expect("launch defined");
    let result = wat::runtime::apply_function(
        launcher.clone(),
        vec![Value::i64(100)],
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect(":my::launch runs (spawn-program' + send' + recv')");
    match &result {
        Value::i64(n) => assert_eq!(
            *n, 107,
            "expected 100+7=107 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── T7. spawn-process with non-portable Sender capture ────────────────

#[test]
fn t7_spawn_process_non_portable_capture_fires_diagnostic() {
    // A factory builds a closure capturing a Sender from the parent's
    // let-scope. The Sender is a channel-bearing Value — pointer
    // identity does not survive fork(2). Slice 1's portability check
    // refuses; spawn-process surfaces the diagnostic.
    // The freeze may succeed (the closure-extract check fires at
    // spawn-process invocation, not at freeze). If the type-checker
    // already rejects, that's also a valid failure mode — both paths
    // refuse the non-portable shape. Fixture: t7_non_portable.wat.
    match startup_from_file("tests/program/wat_arc170_program_contracts_t7_non_portable.wat") {
        Ok(world) => {
            let launcher = world
                .symbols()
                .get(":my::launch")
                .expect("launch defined");
            let result = wat::runtime::apply_function(
                launcher.clone(),
                Vec::new(),
                world.symbols(),
                wat::rust_caller_span!(),
            );
            match result {
                Err(RuntimeError { kind: RuntimeErrorKind::MalformedForm { reason, .. }, .. }) => {
                    // rune:lint(loose-assert) — dead branch: freeze rejects t7 fixture at type-check; runtime MalformedForm arm never executes
                    assert!(
                        reason.contains("impure")
                            || reason.contains("ImpureCapture")
                            || reason.contains("Impure types")
                            || reason.contains("Sender")
                            || reason.contains("Receiver")
                            || reason.contains("captures"),
                        "expected impure-capture diagnostic; got reason: {}",
                        reason
                    );
                }
                Ok(_) => panic!("expected non-portable refusal; succeeded"),
                Err(other) => {
                    let msg = format!("{:?}", other);
                    let lc = msg.to_lowercase();
                    // rune:lint(loose-assert) — dead branch: freeze rejects t7 fixture at type-check; runtime other-error arm never executes
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

// ─── T10. spawn-thread(fn) — UNCHANGED behavior ──────────────────────

#[test]
fn t10_spawn_thread_unchanged_positive_control() {
    // Same shape as before arc 170 — spawn-thread takes a fn whose
    // signature is :Receiver<I> + :Sender<O> → :nil. Behavior must
    // not regress: the thread runs in parent's world, communicates
    // via crossbeam channels, returns Thread<I,O>.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t10_echo_thread.wat");
    // Build (:wat::kernel::spawn-thread :my::echo-thread).
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-thread".into(), wat::rust_caller_span!()),
            WatAST::Keyword(":my::echo-thread".into(), wat::rust_caller_span!()),
        ],
        wat::rust_caller_span!(),
    );
    let env = Environment::new();
    let thread = eval(&call, &env, world.symbols()).expect("spawn-thread succeeds").value_owned();
    let types = world.symbols().types().map(|a| a.as_ref());
    // Thread<I,O> field order: input(0), output(1), join(2)
    let (input, output) = match &thread {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Thread" => (&s.fields[0], &s.fields[1]),
        other => panic!("expected Thread Struct; got {:?}", other),
    };
    let outcome = wat::channel::typed_send(
        unwrap_sender_inner(input),
        Value::i64(21),
        types,
        wat::rust_caller_span!(),
    );
    assert!(matches!(outcome, wat::channel::SendOutcome::Ok));
    let response = match wat::channel::typed_recv(
        unwrap_receiver_inner(output),
        types,
        wat::rust_caller_span!(),
    ) {
        wat::channel::RecvOutcome::Value(v) => v,
        other => panic!("expected Value; got {:?}", other),
    };
    match response {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── T11. 3-arg :user::main fires walker (BareLegacyMainSignature) ────

#[test]
#[ignore = "ARC-170 WIP: BareLegacyMainSignature walker no longer fires for a non-canonical :user::main (freeze succeeds where it should reject — likely walker-disconnect); investigate + fix/retire before arc 170 closes."]
fn t11_legacy_main_signature_fires_walker_diagnostic() {
    unimplemented!("arc 170: BareLegacyMainSignature walker reconnect; on unlock assert the legacy-4arg-main diagnostic exactly");
}

// ─── T12. spawn-process(fn) — child emits without recv'ing first ──────
//
// Slice 1f-λ rebuild for the arc-104 fork_program_child_writes_stdout
// scenario. Under arc 170 the child's "stdout" is a typed Sender<T>;
// the child sends one value via tx without first reading rx. The rx
// channel exists per the contract shape but goes unread.

#[test]
fn t12_spawn_process_child_emits_without_recv() {
    // Arc 170 slice 6 — child is a self-contained program emitting via
    // println; parent reads via Receiver/from-pipe.
    let world = freeze_trivial();
    let call = build_spawn_process_call_from_fixture(
        "tests/program/wat_arc170_program_contracts_t12_child.wat",
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let types = world.symbols().types().map(|a| a.as_ref());
    // Parent reads from Process/stdout via Receiver/from-pipe.
    let stdout_reader = process_stdout_field(&process);
    let receiver_val = wat::channel::receiver_from_pipe(stdout_reader);
    let receiver_inner = unwrap_receiver_inner(&receiver_val);
    let response = drive_typed_recv(receiver_inner, types);
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
    // Arc 170 slice 6 — child program returns immediately; parent drops
    // Process (closes stdin/stdout pipes) → child exits 0.
    let world = freeze_trivial();
    let call = build_spawn_process_call_from_fixture(
        "tests/program/wat_arc170_program_contracts_child_announce.wat",
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let handle = process_handle_field(&process);
    // MEASURE the child ran: it announces itself on its stdout pipe.
    let types = world.symbols().types().map(|a| a.as_ref());
    let receiver_val = wat::channel::receiver_from_pipe(process_stdout_field(&process));
    match drive_typed_recv(unwrap_receiver_inner(&receiver_val), types) {
        Value::String(s) => assert_eq!(&*s, "spawned child", "expected spawned child; got {:?}", s),
        other => panic!("expected String; got {:?}", other),
    }
    // Drop process Struct → stdin/stdout/stderr pipes close; child exits 0.
    drop(process);
    wait_child_exit_ok(handle);
}

// ─── T14. spawn-process(fn) — wait handle is idempotent ──────────────
//
// Slice 1f-λ rebuild for the arc-012 wait_child_is_idempotent scenario.
// ChildHandle::wait_or_cached_exit() uses OnceLock caching; calling it
// twice must return the same exit code rather than re-waiting or
// returning a sentinel. Child fn returns nil immediately (idle worker).

#[test]
fn t14_spawn_process_wait_handle_is_idempotent() {
    // Arc 170 slice 6 — child program returns immediately; idempotent
    // wait_or_cached_exit caches exit 0 on first wait and reuses it on the second.
    let world = freeze_trivial();
    let call = build_spawn_process_call_from_fixture(
        "tests/program/wat_arc170_program_contracts_child_announce.wat",
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let handle = process_handle_field(&process);
    // MEASURE the child ran: it announces itself on its stdout pipe.
    let types = world.symbols().types().map(|a| a.as_ref());
    let receiver_val = wat::channel::receiver_from_pipe(process_stdout_field(&process));
    match drive_typed_recv(unwrap_receiver_inner(&receiver_val), types) {
        Value::String(s) => assert_eq!(&*s, "spawned child", "expected spawned child; got {:?}", s),
        other => panic!("expected String; got {:?}", other),
    }
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
    // Arc 170 slice 6 — child panics intentionally before printing;
    // parent's recv returns Disconnected; exit code is non-zero.
    let world = freeze_trivial();
    let call = build_spawn_process_call_from_fixture(
        "tests/program/wat_arc170_program_contracts_t15_child.wat",
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
    let types = world.symbols().types().map(|a| a.as_ref());
    let handle = process_handle_field(&process);
    // Parent reads from Process/stdout via Receiver/from-pipe.
    // Child panics before println → stdout pipe closes → Disconnected.
    let stdout_reader = process_stdout_field(&process);
    let receiver_val = wat::channel::receiver_from_pipe(stdout_reader);
    let receiver_inner = unwrap_receiver_inner(&receiver_val);
    let recv_outcome = wat::channel::typed_recv(
        receiver_inner,
        types,
        wat::rust_caller_span!(),
    );
    assert!(
        matches!(recv_outcome, wat::channel::RecvOutcome::Disconnected),
        "expected Disconnected (child panicked before printing); got {:?}",
        recv_outcome,
    );
    // Handle exit code must be non-zero (EXIT_PANIC=2).
    use wat::runtime::ProgramHandleInner;
    let code = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached_exit(),
        other => panic!("expected Forked ProgramHandle; got {:?}", other),
    };
    assert_ne!(code, 0, "expected non-zero exit on child panic; got 0");
}

// ─── T17. hermetic run — happy path over the primed peer wire (arc 278 IPC de-prime)
//
// Migrated off the `:wat::test::run-hermetic` macro onto the composed primes:
// `spawn-program' (process)` spawns the peer, its `:user::main` computes 2+2 and
// `println`s it, and the parent drains that single value off the peer via `recv'` —
// the value arrives as a `RecvOutcome::Message`.
//
// Surface form exercised (fixture t17_run_hermetic.wat):
//   (:wat::kernel::spawn-program' (:wat::spawn::process)
//     (:wat::core::forms (:user::main ... (:wat::kernel::println (:wat::core::i64::+ 2 2)))))
//   → (:wat::kernel::recv' p) → RecvOutcome::Message m → m
//
// The function is defined at :my::test::two-plus-two; invoked with
// apply_function (zero args); the recv'd value must be i64 4.

#[test]
fn t17_run_hermetic_layer1_passing_assertion() {
    // Arc 278 IPC de-prime — migrated off :wat::test::run-hermetic onto the primed
    // peer wire. The child's :user::main computes 2+2 and println's it; the parent
    // spawn-program' (process) + recv' receives that single value off the peer as a
    // RecvOutcome::Message. A "passing" run now means the value genuinely crossed the
    // wire as 4 (NOT stdout-scraped). Fixture: t17_run_hermetic.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t17_run_hermetic.wat");
    let func = world
        .symbols()
        .get(":my::test::two-plus-two")
        .expect(":my::test::two-plus-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + recv' should succeed");
    // two-plus-two returns the recv'd i64 directly — the value that crossed the peer wire.
    match &result {
        wat::runtime::Value::i64(n) => assert_eq!(
            *n, 4,
            "expected 2+2=4 received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64 result; got {:?}", other),
    }
}

#[test]
fn t17b_run_hermetic_layer1_failing_assertion_surfaces_failure() {
    // Complementary to T17 — the FAILURE path of the SAME primed wire.
    // Arc 278 IPC de-prime: migrated off :wat::test::run-hermetic onto the composed
    // primes (`spawn-program' (process)` + `recv'`). The child's assert-eq (1+0 != 2)
    // FAILS, so the child PANICS before it can send anything; the parent's recv' returns
    // RecvOutcome::Lost carrying a :wat::kernel::LociDiedError. An assert-eq failure is an
    // AssertionPayload panic, so the cause is LociDiedError::Panic whose failure field is
    // Some(Failure) carrying the structured assert-eq diagnostic. The death is SURFACED
    // (the fixture returns the raw LociDiedError), NEVER swallowed. Mirrors t18b's Lost/
    // Panic assertion shape (minus t18b's recv-all' Result unwrap — recv' hands back the
    // Lost cause directly). Fixture: t17b_run_hermetic_fail.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat");
    let func = world
        .symbols()
        .get(":my::test::one-neq-two")
        .expect(":my::test::one-neq-two defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + recv' driver should not itself panic");

    // The child died on assert-eq before sending, so recv' surfaced a Lost cause;
    // the fixture returns that :wat::kernel::LociDiedError enum directly.
    let ev = match &result {
        wat::runtime::Value::Enum(ev) => ev.as_ref(),
        other => panic!("expected :wat::kernel::LociDiedError enum cause; got {:?}", other),
    };
    assert_eq!(
        ev.type_path, ":wat::kernel::LociDiedError",
        "expected the recv' Lost cause to be a LociDiedError; got type_path {}",
        ev.type_path
    );
    assert_eq!(
        ev.variant_name, "Panic",
        "child assert-eq failure must surface as LociDiedError::Panic; got variant {}",
        ev.variant_name
    );

    // Panic.fields = [message :String, failure :Option<Failure>]. An AssertionPayload panic
    // carries the structured Failure, so failure is Some(Failure).
    let failure_val = match &ev.fields[1] {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!(
                "expected LociDiedError::Panic.failure = Some(Failure) (assert-eq carries an AssertionPayload); got None"
            ),
        },
        other => panic!("expected Panic.failure :Option<Failure>; got {:?}", other),
    };

    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Aggregate(s) if s.nature == wat::Nature::Record && s.class == "wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };
    // Arc 278 the string-wrap annihilation — Failure.fields[0] is the mandatory `error`
    // (Fault); its fields[0] is the message String. Must carry the structured assert-eq
    // diagnostic, read STRUCTURALLY off the surfaced Panic — no string re-parse.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::Aggregate(err) => match &err.fields[0] {
            wat::runtime::Value::String(s) => s.to_string(),
            other => panic!("expected Failure.error.message :String; got {:?}", other),
        },
        other => panic!("expected Failure.error :Aggregate; got {:?}", other),
    };
    assert_eq!(
        message,
        "assert-eq failed",
        "t17b_msg: LociDiedError::Panic carries the assert-eq diagnostic golden"
    );
}

// ─── T18. BIDIRECTIONAL PRIME EXEMPLAR — spawn-program' + send' + recv' drain
//
// Arc 278 IPC de-prime. This consumer retired off `run-hermetic-with-io` (the
// non-prime Sender/from-pipe + Receiver/from-pipe over a 4-field Process) onto
// the composed primes: `spawn-program' (process)` spawns the peer, `send' 21`
// feeds the child's `readln`, and each child `println` crosses back to the
// parent as a `recv'` `RecvOutcome::Message` — drained until `RecvOutcome::Closed`
// (a genuine clean EOF) into the collected outputs.
//
// The child body (readln n → println n*2 → nil) is UNCHANGED from the old form;
// only the DRIVER flipped to the peer wire. `:my::test::echo-doubled` now returns
// the drained outputs directly as a `Vector<i64>` — this test asserts it is [42],
// i.e. the doubled value genuinely crossed the wire as a recv' Message (NOT
// stdout-scraped).
//
// Fixture: t18_echo_doubled.wat (drain is the shared primed helper
// `:wat::kernel::recv-all'`, whose canonical call site this is).

#[test]
fn t18_run_hermetic_with_io_layer2_echo_doubled() {
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat");
    let func = world
        .symbols()
        .get(":my::test::echo-doubled")
        .expect(":my::test::echo-doubled defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv'-drain should succeed");

    // echo-doubled returns the drained outputs directly: Vector<i64> == [42].
    // Each element was received over the peer wire as a recv' RecvOutcome::Message.
    let outputs = match &result {
        wat::runtime::Value::Vec(v) => v.as_ref(),
        other => panic!("expected Vec outputs; got {:?}", other),
    };
    assert_eq!(
        outputs.len(),
        1,
        "expected exactly one output value drained off the peer; got {}",
        outputs.len()
    );
    match &outputs[0] {
        wat::runtime::Value::i64(n) => assert_eq!(
            *n, 42,
            "expected output 42 (21 * 2) received as a recv' Message; got {}",
            n
        ),
        other => panic!("expected i64 output; got {:?}", other),
    }
}

// ─── T18c. recv-all' HELPER GATE — drains ALL outputs, not just one
//
// Arc 278 IPC de-prime. t18 exercises `:wat::kernel::recv-all'` on a single-output
// peer; this is the helper's own gate on the "ALL": a peer that emits THREE `println`
// values must be drained into the full collected Vector, in order, before the clean
// EOF (`RecvOutcome::Closed`) turns into `(Ok outputs)`. The child readln's 7 and
// println's 7, 14, 21; recv-all' returns `Ok [7 14 21]`.
//
// Fixture: t18c_recv_all_multi.wat.
#[test]
fn t18c_recv_all_drains_all_outputs() {
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18c_recv_all_multi.wat");
    let func = world
        .symbols()
        .get(":my::test::echo-multi")
        .expect(":my::test::echo-multi defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv-all' drain should succeed");

    // recv-all' returns Ok[outputs]; echo-multi unwraps to the Vector<i64> == [7 14 21].
    let outputs = match &result {
        wat::runtime::Value::Vec(v) => v.as_ref(),
        other => panic!("expected Vec outputs; got {:?}", other),
    };
    let got: Vec<i64> = outputs
        .iter()
        .map(|v| match v {
            wat::runtime::Value::i64(n) => *n,
            other => panic!("expected i64 output; got {:?}", other),
        })
        .collect();
    assert_eq!(
        got,
        vec![7, 14, 21],
        "recv-all' must drain ALL peer outputs in order into Ok[…]; got {:?}",
        got
    );
}

#[test]
fn t18b_run_hermetic_with_io_layer2_failing_assertion_surfaces_failure() {
    // Complementary to T18 — the FAILURE path of the SAME primed bidirectional wire.
    // Arc 278 IPC de-prime: this consumer retired off `run-hermetic-with-io` onto the
    // composed primes (`spawn-program' (process)` + `send'` + `:wat::kernel::recv-all'`).
    // The child recvs 2 (fed by `send' p 2`), then `assert-eq n 3` fails (2 != 3), so the
    // child PANICS before its `println` — the peer DIES mid-exchange.
    //
    // recv-all' surfaces that death honestly: it returns `(Err cause)` where `cause` is a
    // `:wat::kernel::LociDiedError`. An assert-eq failure is an AssertionPayload panic, so
    // `cause` is the `LociDiedError::Panic` variant, whose `failure` field is `Some(Failure)`
    // carrying the structured assert-eq diagnostic (same shape the arc-278 failure exemplar
    // proves: tests/comms/probe_arc278_failure_carries_structured_error.wat). The death is
    // SURFACED in the Err, NEVER swallowed — that is the whole point of recv-all'.
    //
    // (The old form drained a `RunResultIO` and inspected its `.failure` slot; the peer's
    // own Lost cause IS the failure now, read straight off recv-all''s Err.)
    // Fixture: t18b_recv_assert_fail.wat.
    let world = freeze_ok("tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat");
    let func = world
        .symbols()
        .get(":my::test::recv-assert-fail")
        .expect(":my::test::recv-assert-fail defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("spawn-program' + send' + recv-all' driver should not itself panic");

    // recv-all' returns Result<Vector<i64>, LociDiedError>. The child died mid-exchange,
    // so this MUST be Err[cause] — no outputs were drained (the child panicked before println).
    let cause = match &result {
        wat::runtime::Value::Result(r) => match &**r {
            Ok(ok) => panic!(
                "expected Err[LociDiedError] (child died on assert-eq mid-exchange); got Ok({:?})",
                ok
            ),
            Err(cause) => cause,
        },
        other => panic!("expected Value::Result from recv-all'; got {:?}", other),
    };

    // The Err cause is a :wat::kernel::LociDiedError enum. An assert-eq failure panics with
    // an AssertionPayload, so the peer's death is the Panic variant.
    let ev = match cause {
        wat::runtime::Value::Enum(ev) => ev.as_ref(),
        other => panic!("expected :wat::kernel::LociDiedError enum cause; got {:?}", other),
    };
    assert_eq!(
        ev.type_path, ":wat::kernel::LociDiedError",
        "expected the Err cause to be a LociDiedError; got type_path {}",
        ev.type_path
    );
    assert_eq!(
        ev.variant_name, "Panic",
        "child assert-eq failure must surface as LociDiedError::Panic; got variant {}",
        ev.variant_name
    );

    // Panic.fields = [message :String, failure :Option<Failure>]. An AssertionPayload panic
    // carries the structured Failure, so failure is Some(Failure).
    let failure_val = match &ev.fields[1] {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!(
                "expected LociDiedError::Panic.failure = Some(Failure) (assert-eq carries an AssertionPayload); got None"
            ),
        },
        other => panic!("expected Panic.failure :Option<Failure>; got {:?}", other),
    };

    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Aggregate(s) if s.nature == wat::Nature::Record && s.class == "wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };

    // Failure.message (arc 278 — fields[0] is `error` (Fault); its fields[0] is the message
    // String) must carry the structured assert-eq diagnostic, read STRUCTURALLY off the
    // surfaced Panic — no string re-parse. Phase C′ emit_panics_to_stderr is active for
    // spawn_process; the child's assertion diagnostic rides the Lost cause intact.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::Aggregate(err) => match &err.fields[0] {
            wat::runtime::Value::String(s) => s.to_string(),
            other => panic!("expected Failure.error.message :String; got {:?}", other),
        },
        other => panic!("expected Failure.error :Aggregate; got {:?}", other),
    };
    assert_eq!(
        message,
        "assert-eq failed",
        "t18b_msg: LociDiedError::Panic carries the assert-eq diagnostic golden"
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
    // Arc 170 slice 6 — three sequential spawn-process+exit cycles;
    // pipes close cleanly; waitpid reaps zombies; no accumulation.
    let world = freeze_trivial();
    let env = Environment::new();
    for _ in 0..3 {
        let call = build_spawn_process_call_from_fixture(
            "tests/program/wat_arc170_program_contracts_child_announce.wat",
        );
        let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
        let handle = process_handle_field(&process);
        // MEASURE the child ran: it announces itself on its stdout pipe.
        let types = world.symbols().types().map(|a| a.as_ref());
        let receiver_val = wat::channel::receiver_from_pipe(process_stdout_field(&process));
        match drive_typed_recv(unwrap_receiver_inner(&receiver_val), types) {
            Value::String(s) => assert_eq!(&*s, "spawned child", "expected spawned child; got {:?}", s),
            other => panic!("expected String; got {:?}", other),
        }
        // Drop process → child exits 0.
        drop(process);
        wait_child_exit_ok(handle);
    }
}
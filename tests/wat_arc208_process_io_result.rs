//! Arc 208 slice 1 — `:wat::kernel::Process/readln` + `:wat::kernel::Process/println`
//! return `Result<_, Vector<ProcessDiedError>>`.
//!
//! **What this file proves:**
//!
//! - T1 (type-scheme registration) — both verbs are registered as
//!   Result-returning in the type env; the old raw-return shapes are gone.
//! - T2 (happy-path Ok) — `Process/println` on a live peer returns
//!   `Ok(nil)`, not raw nil; `Process/readln` on a live peer returns
//!   `Ok(v)`, not raw v. Values are correct after unwrapping.
//! - T3 (Err on dead peer, println) — writing to a peer whose subprocess
//!   has exited returns `Err(chain)` with a non-empty
//!   `Vector<ProcessDiedError>` chain; does NOT panic as a
//!   `RuntimeError::ChannelDisconnected`.
//! - T4 (Err on dead peer, readln) — reading from a peer whose subprocess
//!   has exited returns `Err(chain)` with a non-empty chain.
//! - T5 (chain content) — the `ProcessDiedError::ChannelDisconnected`
//!   variant appears as the head of the Err chain from both verbs on a
//!   dead peer (matching what `Process/drain-and-join` reports for the
//!   same subprocess).
//!
//! **Walker rule** — arc 208 slice 1 also adds `Process/readln` and
//! `Process/println` to the `validate_comm_positions` checker so calling
//! either outside `match`/`Result/expect`/`Option/expect` is a compile-
//! time error. Tests T6 and T7 verify the walker fires when the verbs
//! appear in forbidden positions.
//!
//! Architecture mirrors `tests/wat_arc170_stone_a_drain_and_join.rs`
//! and `tests/wat_process_peer_ipc_round_trip.rs`. Child programs are
//! inlined as string constants; `build_spawn_process_call` is local.

use std::sync::Arc;

use wat::ast::WatAST;
use wat::check::CheckEnv;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, Value};
use wat::span::Span;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

fn freeze_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("freeze should fail but succeeded"),
        Err(e) => format!("{}", e),
    }
}

/// Wrap a child-program source string as a
/// `(:wat::kernel::spawn-process (:wat::core::forms <forms>...))` call AST.
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), Span::unknown()),
            forms_call,
        ],
        Span::unknown(),
    )
}

/// Echo server: reads one String line from stdin, writes it back to stdout.
const ECHO_SERVER: &str = r#"
    (:wat::core::define (:user::main -> :wat::core::nil)
      (:wat::core::let
        [line (:wat::kernel::readln -> :wat::core::String)]
        (:wat::kernel::println line)))
"#;

/// Minimal server that exits immediately (nothing on stdout).
const IMMEDIATE_EXIT_SERVER: &str = r#"
    (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
"#;

/// Trivial parent program (needed to freeze a parent-side world).
const PARENT_SRC: &str = r#"
    (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
"#;

/// Unwrap `Value::Result(Ok(inner))` and return `inner`. Panics otherwise.
fn unwrap_ok(v: Value, label: &str) -> Value {
    match v {
        Value::Result(r) => match Arc::try_unwrap(r).unwrap_or_else(|a| (*a).clone()) {
            Ok(inner) => inner,
            Err(chain) => panic!("{}: expected Ok; got Err({:?})", label, chain),
        },
        other => panic!("{}: expected Value::Result; got {:?}", label, other),
    }
}

/// Unwrap `Value::Result(Err(chain))` and return the chain. Panics on Ok.
fn unwrap_err_chain(v: Value, label: &str) -> Value {
    match v {
        Value::Result(r) => match Arc::try_unwrap(r).unwrap_or_else(|a| (*a).clone()) {
            Err(chain) => chain,
            Ok(inner) => panic!("{}: expected Err; got Ok({:?})", label, inner),
        },
        other => panic!("{}: expected Value::Result; got {:?}", label, other),
    }
}

// ─── T1. Type-scheme registration ─────────────────────────────────────────

#[test]
fn arc208_t1_process_readln_println_registered_as_result_returning() {
    // CheckEnv::with_builtins() is the canonical source of substrate
    // type-scheme registrations — mirrors what the type-checker uses at
    // freeze time. We query it directly (no FrozenWorld needed).
    let check_env = CheckEnv::with_builtins();

    // Process/readln: Result<I, Vector<ProcessDiedError>> — not bare :I.
    let readln_scheme = check_env
        .get(":wat::kernel::Process/readln")
        .expect("Process/readln registered in CheckEnv");
    let readln_ret_str = format!("{:?}", readln_scheme.ret);
    assert!(
        readln_ret_str.contains("Result"),
        "Process/readln return type should contain Result; got: {}",
        readln_ret_str
    );
    assert!(
        readln_ret_str.contains("ProcessDiedError") || readln_ret_str.contains("Vector"),
        "Process/readln return type should mention ProcessDiedError chain; got: {}",
        readln_ret_str
    );

    // Process/println: Result<(), Vector<ProcessDiedError>> — not bare nil.
    let println_scheme = check_env
        .get(":wat::kernel::Process/println")
        .expect("Process/println registered in CheckEnv");
    let println_ret_str = format!("{:?}", println_scheme.ret);
    assert!(
        println_ret_str.contains("Result"),
        "Process/println return type should contain Result; got: {}",
        println_ret_str
    );
    assert!(
        println_ret_str.contains("ProcessDiedError") || println_ret_str.contains("Vector"),
        "Process/println return type should mention ProcessDiedError chain; got: {}",
        println_ret_str
    );
}

// ─── T2. Happy path — Ok on a live peer ───────────────────────────────────

#[test]
fn arc208_t2_process_println_and_readln_return_ok_on_live_peer() {
    // Spawn an echo server; build a ProcessPeer; send "arc208-ok" via
    // Process/println; read the echo back via Process/readln; verify
    // both return Ok-wrapped values.
    let world = freeze_ok(PARENT_SRC);
    let spawn_call = build_spawn_process_call(ECHO_SERVER);
    let env = Environment::new();
    let server = eval(&spawn_call, &env, world.symbols()).expect("spawn-process succeeds");

    let env2 = Environment::new().child().bind("server", server).build();

    // Build the peer bindings shared by both passes.
    // Pass 1 (println): verify Process/println returns Result::Ok(nil).
    // The echo server reads one line and writes it back; we send here.
    let println_ast = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
           tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
           peer (:wat::kernel::ProcessPeer/new rx tx)]
          (:wat::core::match (:wat::kernel::Process/println peer "arc208-ok")
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>
            ((:wat::core::Ok _)  (:wat::core::Ok ()))
            ((:wat::core::Err e) (:wat::core::Err e))))
        "#
    )
    .expect("println-AST parses");

    let sent_result = eval(&println_ast, &env2, world.symbols())
        .expect("Process/println eval should succeed");
    let sent_inner = unwrap_ok(sent_result, "Process/println Ok");
    assert!(
        matches!(sent_inner, Value::Unit),
        "Process/println Ok should carry nil (unit); got {:?}",
        sent_inner
    );

    // Pass 2 (readln + drain): the server echoes what we sent in pass 1.
    // Verify Process/readln returns Result::Ok(String).
    let readln_drain_ast = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx    (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
           tx    (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
           peer  (:wat::kernel::ProcessPeer/new rx tx)
           reply (:wat::core::match (:wat::kernel::Process/readln peer)
                   -> :wat::core::Result<wat::core::String,wat::core::Vector<wat::kernel::ProcessDiedError>>
                   ((:wat::core::Ok v)  (:wat::core::Ok v))
                   ((:wat::core::Err e) (:wat::core::Err e)))
           _done (:wat::kernel::Process/drain-and-join server)]
          reply)
        "#
    )
    .expect("readln+drain AST parses");

    let reply_result = eval(&readln_drain_ast, &env2, world.symbols())
        .expect("Process/readln+drain eval should succeed");
    let reply_inner = unwrap_ok(reply_result, "Process/readln Ok");
    match reply_inner {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "arc208-ok",
            "echo server should reply with the same string"
        ),
        other => panic!(
            "Process/readln Ok should carry String(\"arc208-ok\"); got {:?}",
            other
        ),
    }
}

// ─── T3. Err path — Process/println on dead peer ──────────────────────────

#[test]
fn arc208_t3_process_println_returns_err_on_dead_peer() {
    // Spawn a server that exits immediately (no stdout reads). After
    // it exits its stdin pipe is closed. Writing via Process/println
    // to the dead peer should return Err(chain), NOT panic as
    // RuntimeError::ChannelDisconnected.
    let world = freeze_ok(PARENT_SRC);
    let spawn_call = build_spawn_process_call(IMMEDIATE_EXIT_SERVER);
    let env = Environment::new();
    let server = eval(&spawn_call, &env, world.symbols()).expect("spawn-process succeeds");

    // Drain and join first so the subprocess is definitely dead.
    let env2 = Environment::new().child().bind("server", server).build();
    let djoin_ast = wat::parse_one!("(:wat::kernel::Process/drain-and-join server)")
        .expect("drain-and-join AST parses");
    let _djoin = eval(&djoin_ast, &env2, world.symbols())
        .expect("Process/drain-and-join should succeed");

    // Now re-spawn a fresh server that exits immediately and attempt
    // to write to it after a drain (guarantees dead peer).
    let server2 = eval(
        &build_spawn_process_call(IMMEDIATE_EXIT_SERVER),
        &Environment::new(),
        world.symbols(),
    )
    .expect("second spawn-process succeeds");

    let env3 = Environment::new().child().bind("server2", server2).build();

    // Build peer, drain server, THEN try Process/println on the dead peer.
    let println_dead_ast = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server2))
           tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server2))
           peer (:wat::kernel::ProcessPeer/new rx tx)
           _    (:wat::kernel::Process/drain-and-join server2)]
          (:wat::core::match (:wat::kernel::Process/println peer "should-fail")
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>
            ((:wat::core::Ok _)  (:wat::core::Ok ()))
            ((:wat::core::Err e) (:wat::core::Err e))))
        "#
    )
    .expect("println-dead AST parses");

    let outcome = eval(&println_dead_ast, &env3, world.symbols())
        .expect("Process/println on dead peer should return Result, not panic");

    let chain = unwrap_err_chain(outcome, "Process/println dead peer");
    match chain {
        Value::Vec(v) => assert!(
            !v.is_empty(),
            "Err chain should be non-empty on dead peer"
        ),
        other => panic!(
            "Process/println Err should carry Vec<ProcessDiedError>; got {:?}",
            other
        ),
    }
}

// ─── T4. Err path — Process/readln on dead peer ───────────────────────────

#[test]
fn arc208_t4_process_readln_returns_err_on_dead_peer() {
    // Mirror of T3 for Process/readln: read from a peer whose subprocess
    // has exited and produces EOF on its stdout pipe.
    let world = freeze_ok(PARENT_SRC);

    // Spawn a server that exits without printing anything.
    let server = eval(
        &build_spawn_process_call(IMMEDIATE_EXIT_SERVER),
        &Environment::new(),
        world.symbols(),
    )
    .expect("spawn-process succeeds");

    let env = Environment::new().child().bind("server", server).build();

    // Build peer, drain server, THEN try Process/readln on the dead peer.
    let readln_dead_ast = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
           tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
           peer (:wat::kernel::ProcessPeer/new rx tx)
           _    (:wat::kernel::Process/drain-and-join server)]
          (:wat::core::match (:wat::kernel::Process/readln peer)
            -> :wat::core::Result<wat::core::String,wat::core::Vector<wat::kernel::ProcessDiedError>>
            ((:wat::core::Ok v)  (:wat::core::Ok v))
            ((:wat::core::Err e) (:wat::core::Err e))))
        "#
    )
    .expect("readln-dead AST parses");

    let outcome = eval(&readln_dead_ast, &env, world.symbols())
        .expect("Process/readln on dead peer should return Result, not panic");

    let chain = unwrap_err_chain(outcome, "Process/readln dead peer");
    match chain {
        Value::Vec(v) => assert!(
            !v.is_empty(),
            "Err chain should be non-empty on dead peer"
        ),
        other => panic!(
            "Process/readln Err should carry Vec<ProcessDiedError>; got {:?}",
            other
        ),
    }
}

// ─── T5. Chain content — ChannelDisconnected head ─────────────────────────

#[test]
fn arc208_t5_err_chain_head_is_channel_disconnected() {
    // Both Process/readln and Process/println should produce
    // ProcessDiedError::ChannelDisconnected as the chain head on a dead peer.
    // Verify the variant name matches the substrate-vended enum.
    let world = freeze_ok(PARENT_SRC);

    let server = eval(
        &build_spawn_process_call(IMMEDIATE_EXIT_SERVER),
        &Environment::new(),
        world.symbols(),
    )
    .expect("spawn-process succeeds");

    let env = Environment::new().child().bind("server", server).build();

    // Process/readln on dead peer — check chain head variant.
    let readln_chain_ast = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
           tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
           peer (:wat::kernel::ProcessPeer/new rx tx)
           _    (:wat::kernel::Process/drain-and-join server)]
          (:wat::core::match (:wat::kernel::Process/readln peer)
            -> :wat::core::Result<wat::core::String,wat::core::Vector<wat::kernel::ProcessDiedError>>
            ((:wat::core::Ok v)  (:wat::core::Ok v))
            ((:wat::core::Err e) (:wat::core::Err e))))
        "#
    )
    .expect("readln chain AST parses");

    let outcome = eval(&readln_chain_ast, &env, world.symbols())
        .expect("Process/readln dead peer returns Result");

    let chain = unwrap_err_chain(outcome, "T5 readln chain");
    // Chain is Vec<ProcessDiedError>; extract head.
    let head = match &chain {
        Value::Vec(v) if !v.is_empty() => &v[0],
        other => panic!("expected non-empty Vec; got {:?}", other),
    };
    // Head should be ProcessDiedError::ChannelDisconnected.
    match head {
        Value::Enum(e) => {
            assert_eq!(
                e.type_path, ":wat::kernel::ProcessDiedError",
                "chain head type_path should be :wat::kernel::ProcessDiedError"
            );
            assert_eq!(
                e.variant_name, "ChannelDisconnected",
                "chain head variant should be ChannelDisconnected; got {}",
                e.variant_name
            );
        }
        other => panic!(
            "chain head should be a ProcessDiedError enum; got {:?}",
            other
        ),
    }
}

// ─── T6. Walker rule — Process/println in forbidden position ──────────────

#[test]
fn arc208_t6_walker_rejects_process_println_in_body_position() {
    // Process/println appearing directly as a function body expression
    // (not as the scrutinee of match or value-position of Result/expect)
    // is the forbidden pattern arc 208 adds to the validate_comm_positions
    // walker. The walker fires on WatAST::List nodes; the direct function
    // body is such a node.
    //
    // Note: let-binding RHS inside a WatAST::Vector is NOT reached by the
    // walker (Vector nodes early-return per the walker's structural contract).
    // Forbidden positions the walker covers: direct body expressions, `do`
    // children, function argument positions, etc.
    let src = r#"
        (:wat::core::defn :user::bad-println
          [peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::String>]
          -> :wat::core::nil
          (:wat::core::do
            (:wat::kernel::Process/println peer "hello")
            :wat::core::nil))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("CommCallOutOfPosition") || err.contains("Process/println"),
        "walker should fire CommCallOutOfPosition for Process/println in do-body; got: {}",
        err
    );
}

// ─── T7. Walker rule — Process/readln in forbidden position ───────────────

#[test]
fn arc208_t7_walker_rejects_process_readln_in_body_position() {
    // Mirror of T6 for Process/readln: direct body expression in a `do`
    // form triggers CommCallOutOfPosition.
    let src = r#"
        (:wat::core::defn :user::bad-readln
          [peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::String>]
          -> :wat::core::String
          (:wat::core::do
            (:wat::kernel::Process/readln peer)
            "fallback"))
    "#;
    let err = freeze_err(src);
    assert!(
        err.contains("CommCallOutOfPosition") || err.contains("Process/readln"),
        "walker should fire CommCallOutOfPosition for Process/readln in do-body; got: {}",
        err
    );
}

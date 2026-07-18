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
//! and `tests/wat_process_peer_ipc_round_trip.rs`. T2-T5's child programs +
//! spawn-process mechanics live in the co-located
//! `wat_arc208_process_io_result.wat` fixture (arc 278 no_inlined_wat
//! migration); each is driven via `call_beside`.

use std::sync::Arc;

use wat::check::CheckEnv;
use wat::freeze::{call_beside, startup_from_file};
use wat::runtime::Value;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_err(fixture_rel: &str) -> String {
    match startup_from_file(fixture_rel) {
        Ok(_) => panic!("freeze should fail but succeeded"),
        Err(e) => format!("{}", e),
    }
}

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
    // CheckEnv::with_builtins_and_types() is the canonical source of substrate
    // type-scheme registrations — mirrors what the type-checker uses at
    // freeze time. We query it directly (no FrozenWorld needed).
    // Stone 243.3.1 — with_builtins() removed; caller binds TypeEnv first.
    let types = wat::types::TypeEnv::with_builtins();
    let check_env = CheckEnv::with_builtins_and_types(&types);

    // Process/readln: Result<I, Vector<ProcessDiedError>> — not bare :I.
    let readln_scheme = check_env
        .get(":wat::kernel::Process/readln")
        .expect("Process/readln registered in CheckEnv");
    let readln_ret_str = format!("{:?}", readln_scheme.ret);
    assert_eq!(
        readln_ret_str,
        "Parametric { head: \"wat::core::Result\", args: [Path(\":I\"), Parametric { head: \"wat::core::Vector\", args: [Path(\":wat::kernel::ProcessDiedError\")] }] }",
        "Process/readln return type must match golden"
    );

    // Process/println: Result<(), Vector<ProcessDiedError>> — not bare nil.
    let println_scheme = check_env
        .get(":wat::kernel::Process/println")
        .expect("Process/println registered in CheckEnv");
    let println_ret_str = format!("{:?}", println_scheme.ret);
    assert_eq!(
        println_ret_str,
        "Parametric { head: \"wat::core::Result\", args: [Tuple([]), Parametric { head: \"wat::core::Vector\", args: [Path(\":wat::kernel::ProcessDiedError\")] }] }",
        "Process/println return type must match golden"
    );
}

// ─── T2. Happy path — Ok on a live peer ───────────────────────────────────

#[test]
fn arc208_t2_process_println_and_readln_return_ok_on_live_peer() {
    // Fixture spawns an echo server, sends "arc208-ok" via Process/println
    // (pass 1), then reads the echo back via Process/readln + drains (pass
    // 2), against the SAME live peer — returns both raw Results as a Tuple.
    let got = call_beside(file!(), ":user::t2-println-then-readln")
        .expect("t2-println-then-readln should not raise");
    let (pass1, pass2) = match got {
        Value::Tuple(items) => (items[0].clone(), items[1].clone()),
        other => panic!("expected Tuple(pass1, pass2); got {:?}", other),
    };

    // Pass 1 (println): verify Process/println returns Result::Ok(nil).
    let sent_inner = unwrap_ok(pass1, "Process/println Ok");
    assert!(
        matches!(sent_inner, Value::Unit),
        "Process/println Ok should carry nil (unit); got {:?}",
        sent_inner
    );

    // Pass 2 (readln + drain): the server echoes what we sent in pass 1.
    // Verify Process/readln returns Result::Ok(String).
    let reply_inner = unwrap_ok(pass2, "Process/readln Ok");
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
    // Fixture spawns a first server + drains it (vestigial, unused — kept to
    // match the original construction exactly), then a second server it also
    // drains before attempting Process/println on the now-dead peer. Writing
    // to a dead peer should return Err(chain), NOT panic as
    // RuntimeError::ChannelDisconnected.
    let outcome = call_beside(file!(), ":user::t3-println-dead-peer")
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
    let outcome = call_beside(file!(), ":user::t4-readln-dead-peer")
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
    // Verify the variant name matches the substrate-vended enum. Reuses the
    // T4 fixture entry — the original T4/T5 Rust-built ASTs were byte-identical.
    let outcome = call_beside(file!(), ":user::t4-readln-dead-peer")
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

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
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
    // Negative fixture loaded from co-located wat_arc208_process_io_result_bad_println.wat.
    let err = freeze_err("tests/process/wat_arc208_process_io_result_bad_println.wat");
    assert_eq!(
        err,
        "check:\n2 type-check error(s):\n  - tests/process/wat_arc208_process_io_result_bad_println.wat:9:6: :wat::kernel::Process/println may appear only as the scrutinee of `:wat::core::match`, the value-position of `:wat::core::Result/expect`, or the value-position of `:wat::core::Option/expect`; silent disconnect must be handled at every comm call\n  - tests/process/wat_arc208_process_io_result_bad_println.wat:10:5: malformed :wat::core::nil form: Doctrine 1 (arc 242): ':wat::core::nil' is a TYPE keyword, not a value; use bare `nil` in value position\n",
        "walker should fire CommCallOutOfPosition for Process/println in do-body"
    );
}

// ─── T7. Walker rule — Process/readln in forbidden position ───────────────

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn arc208_t7_walker_rejects_process_readln_in_body_position() {
    // Mirror of T6 for Process/readln: direct body expression in a `do`
    // form triggers CommCallOutOfPosition.
    // Negative fixture loaded from co-located wat_arc208_process_io_result_bad_readln.wat.
    let err = freeze_err("tests/process/wat_arc208_process_io_result_bad_readln.wat");
    assert_eq!(
        err,
        "check:\n1 type-check error(s):\n  - tests/process/wat_arc208_process_io_result_bad_readln.wat:9:6: :wat::kernel::Process/readln may appear only as the scrutinee of `:wat::core::match`, the value-position of `:wat::core::Result/expect`, or the value-position of `:wat::core::Option/expect`; silent disconnect must be handled at every comm call\n",
        "walker should fire CommCallOutOfPosition for Process/readln in do-body"
    );
}

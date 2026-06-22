//! Stone 259-negatives — gold-standard negative IPC framing tests.
//!
//! Two tests prove that the process-peer receive path rejects malformed
//! frames and surfaces the rejection as an observable RuntimeError from
//! `recv'` — NEVER as a hang, NEVER as a silently-wrong decoded value.
//!
//! The true over-cap (1 MiB flood / deadlock) test lives in
//! `tests/probe_overcap_no_deadlock.rs`. The 524,289-byte "1 byte over" test
//! was a dodge: at that size the parent always drains all bytes before TooLarge
//! fires, so `err.recv()` returns quickly (no deadlock). The real deadlock
//! requires a payload large enough that the child is still alive and blocked
//! in `write_all` when TooLarge fires.
//!
//! The two rejection shapes tested here:
//!   1. **truncated** — child emits a partial EDN value (`"{:a 1"`, no closing `}`)
//!      then exits. The partial bytes in the io_uring accumulator match no newline;
//!      on the next read `n == 0` (EOF) → `Err(RecvError::Disconnected)` → same path
//!      → `recv'` raises "process channel disconnected".
//!
//!   2. **anti-smuggle** — child emits `"{:a 1} {:b 2}\n"` (two values on one
//!      physical line, smuggled by writing the `\n` directly via `print-raw'`).
//!      The framer finds the `\n` and checks `edn_frame_status("{:a 1} {:b 2}")`:
//!      `wat_edn::parse_owned` fails with trailing-content error (not Incomplete)
//!      → `EdnFrameStatus::Malformed` → `FrameScan::Frame(end)` → `decode_trusted_wire`
//!      fails → `recv'` raises "recv' EDN decode failed". The trailing `{:b 2}` is
//!      rejected, NOT silently dropped.
//!
//! **Why Rust integration tests (not `.wat` deftest' files)**:
//! `deftest'` expands via `run-thread'` which wraps the body in a WAT thread.
//! When `recv'` raises a RuntimeError inside the thread body, the WAT thread
//! exits cleanly (RuntimeError is a return value, not a Rust panic) WITHOUT
//! sending a crash reason. The outer `recv'` on the thread peer then raises
//! "peer closed / thread exited" — a generic disconnect that loses the
//! original IPC-specific message. `:should-panic` on that generic string is
//! vacuous (any failure produces it). Rust integration tests inspect the
//! EXACT error from the process `recv'` call and assert the IPC-specific
//! substring — non-vacuous by construction.
//!
//! All three tests rely on `print-raw'` (`:wat::kernel::print-raw'`) being
//! registered (Part A). The child program defines `:user::main` that calls
//! `print-raw'` to emit the malformed bytes.
//!
//! Modeled on `tests/probe_supervisor_select_lost.rs`.

use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment};
use wat::span::Span;

// ─── Shared helpers ────────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Build `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program'".into(), Span::unknown()),
            WatAST::List(
                vec![WatAST::Keyword(":wat::spawn::process".into(), Span::unknown())],
                Span::unknown(),
            ),
            forms_call,
        ],
        Span::unknown(),
    )
}

// ─── Test 1 — truncated ───────────────────────────────────────────────────────

/// Child writes a partial EDN value (`"{:a 1"`, no closing `}` or `\n`) then exits.
///
/// No complete frame is ever delivered; EOF arrives while the accumulator holds
/// partial bytes. The `recv` loop sees `n == 0` (EOF) → `Err(RecvError::Disconnected)`
/// → `ProcessPeerBundle::recv` reads err channel (clean exit) →
/// `PeerRecvError::Disconnected` → `recv'` raises "process channel disconnected".
const TRUNCATED_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::kernel::print-raw' "{:a 1"))
"#;

/// Receiving from a child that exits mid-frame (no `\n`) must raise.
///
/// Proves `print-raw'` does NOT add a trailing newline: if it did, the framer
/// would produce Frame("{:a 1"), decode would fail with a parse error (not
/// "disconnected"), and the test would catch the wrong message.
/// The "process channel disconnected" message is ONLY produced by the
/// EOF-before-frame path, which requires the `print-raw'` call to have
/// written 0 newlines.
#[test]
fn truncated_frame_is_rejected_by_recv_prime() {
    let world = freeze_ok("");

    let spawn_call = build_spawn_process_call(TRUNCATED_CHILD_SRC);
    let child = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    let env = Environment::new()
        .child()
        .bind("child", Span::unknown(), child.into())
        .build();

    let recv_call = wat::parse_one!(r#"(:wat::kernel::recv' child)"#)
        .expect("parse recv' call");

    let result = eval(&recv_call, &env, world.symbols());

    match result {
        Ok(tv) => panic!(
            "truncated: recv' must raise on partial frame (no \\n); got value {:?}",
            tv.value_owned()
        ),
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("process channel disconnected"),
                "truncated: recv' raised with unexpected message (expected 'process channel disconnected'); got: {}",
                msg
            );
        }
    }
}

// ─── Test 2 — anti-smuggle ────────────────────────────────────────────────────

/// Child writes `"{:a 1} {:b 2}\n"` (two EDN values on one physical line).
///
/// The `\n` is included in the raw `print-raw'` write — NOT added by the
/// StdOutService (which would frame a single value). The framer hits the `\n`:
/// `edn_frame_status("{:a 1} {:b 2}")` → `wat_edn::parse_owned` fails with
/// trailing-content error (the `{:b 2}` after the first complete map) →
/// `EdnFrameStatus::Malformed` → `FrameScan::Frame(end)` → frame content is
/// `"{:a 1} {:b 2}"` (newline stripped) → `decode_trusted_wire` fails →
/// `recv'` raises "EDN decode failed". The trailing `{:b 2}` is REJECTED,
/// not silently dropped or accepted as a second value.
const ANTI_SMUGGLE_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::kernel::print-raw' "{:a 1} {:b 2}\n"))
"#;

/// Receiving a smuggled second value on one physical line must raise with a decode error.
///
/// The `:b 2` portion is not silently dropped — `decode_trusted_wire` rejects
/// the double-value frame. The assertion checks for "EDN decode failed" to
/// distinguish this from the "process channel disconnected" path (which would
/// indicate a framing-level drop instead of a decode-level rejection — a
/// different and less honest failure mode).
#[test]
fn anti_smuggle_frame_is_rejected_by_recv_prime() {
    let world = freeze_ok("");

    let spawn_call = build_spawn_process_call(ANTI_SMUGGLE_CHILD_SRC);
    let child = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    let env = Environment::new()
        .child()
        .bind("child", Span::unknown(), child.into())
        .build();

    let recv_call = wat::parse_one!(r#"(:wat::kernel::recv' child)"#)
        .expect("parse recv' call");

    let result = eval(&recv_call, &env, world.symbols());

    match result {
        Ok(tv) => panic!(
            "anti-smuggle: recv' must raise on double-value line; got value {:?}",
            tv.value_owned()
        ),
        Err(e) => {
            let msg = format!("{}", e);
            assert!(
                msg.contains("EDN decode failed"),
                "anti-smuggle: recv' raised with unexpected message (expected 'EDN decode failed'); got: {}",
                msg
            );
        }
    }
}

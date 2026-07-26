//! Integration coverage for the canonical body-AST entry path.
//!
//! Arc 278 IPC de-prime: both drivers migrated off the non-prime runners
//! (`:wat::test::run-hermetic` / `:wat::test::run-thread`) onto the PRIMED
//! peer wire (`spawn-program'` + `recv'`), the same shape `run-hermetic'` /
//! `run-thread'` already ride. The inner child bodies (`println "hello"` /
//! `assert-eq 1 2`) are unchanged; only the DRIVER flips.
//!
//! - `ast_entry_prints_hello` (process tier): the child `println`s "hello";
//!   on the primed wire the parent `recv'`s it as a DECODED
//!   `RecvOutcome::Message[m]` — `m` is the native String "hello", NOT the
//!   EDN-quoted stdout scrape (`"\"hello\""`) the old `RunResult/stdout`
//!   read produced. The assertion updates accordingly.
//! - `ast_entry_captures_assertion_failure` (thread tier): the child's
//!   failing `assert-eq` CRASHES the peer → `recv'` returns `Lost[cause]`;
//!   the fixture maps that detected failure to 1 (mirroring the old
//!   `RunResult/failure` Some→1 / None→0 read). Still expects 1.
//!
//! Tests use the `(:my::compute -> :T)` helper + `call_beside_value` for the outer
//! layer; Rust asserts on the returned Value.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

fn unwrap_string(v: Value) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Body-AST entry — happy path (run-hermetic per rules 1+2) ──────────

#[test]
fn ast_entry_prints_hello() {
    // Arc 278 IPC de-prime: migrated from `:wat::test::run-hermetic` +
    // `RunResult/stdout` (an OS-pipe stdout scrape that captured the
    // EDN-quoted `"\"hello\""`) onto `spawn-program'` + `recv'`. The primed
    // wire delivers the child's `println`'d value as a DECODED message, so
    // the parent receives the native String "hello" (no EDN quotes).
    assert_eq!(unwrap_string(run_fn(":my::compute-prints-hello")), "hello");
}

// ─── Body-AST entry — failure surfaces identically (run-thread safe) ───

#[test]
fn ast_entry_captures_assertion_failure() {
    // The body calls assert-eq with mismatched args.
    //
    // Arc 278 IPC de-prime: migrated from `:wat::test::run-thread` +
    // `RunResult/failure` onto `spawn-program' (thread)` + `recv'`. The
    // failing assertion crashes the thread peer → `recv'` returns
    // `Lost[cause]`, which the fixture maps to 1 (the detected failure) —
    // reproducing the old Some→1 / None→0 read. Still expects 1.
    match run_fn(":my::compute-assertion-failure") {
        Value::i64(n) => assert_eq!(n, 1, "expected failure to be detected (1); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

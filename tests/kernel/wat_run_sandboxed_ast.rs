//! Integration coverage for the canonical body-AST entry path —
//! historically `:wat::kernel::run-sandboxed-ast`, now exercised through
//! `:wat::test::run-hermetic` / `:wat::test::run-thread` per arc 170
//! slice 4c-α-ii. The legacy substrate verb still exists (retires in
//! task #310 after the whole 4c-α chain lands); these tests now ride the
//! canonical macros so they share semantics with the rest of the test
//! corpus.
//!
//! Per-site destinations follow FM 7-ter (three-rule classification):
//! - `ast_entry_prints_hello` reads `RunResult/stdout` AND the body calls
//!   `:wat::kernel::println` → rules 1+2 → run-hermetic.
//! - `ast_entry_captures_assertion_failure` reads only `RunResult/failure`
//!   with no stdio activity in the body → run-thread is safe.
//!
//! Arc 170 slice 1f-ζ: outer `:user::main` retired. Tests use
//! `(:my::compute -> :T)` helper + `eval_in_frozen` for the outer
//! layer. Inner programs use canonical nil main + `:wat::kernel::println`.
//! Rust asserts on the Value returned by eval_in_frozen.

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("eval should succeed")
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
    // Arc 170 slice 4c-α-ii: migrated from `:wat::kernel::run-sandboxed-ast`
    // to `:wat::test::run-hermetic`. The body invokes
    // `:wat::kernel::println` and the outer reads `RunResult/stdout` —
    // rules 1+2 of FM 7-ter demand hermetic for accurate stdio capture.
    // :wat::kernel::println EDN-serializes strings with quotes.
    assert_eq!(unwrap_string(run_fn(":my::compute-prints-hello")), "\"hello\"");
}

// ─── Body-AST entry — failure surfaces identically (run-thread safe) ───

#[test]
fn ast_entry_captures_assertion_failure() {
    // The body calls assert-eq with mismatched args; the run-thread
    // driver's join-result Err arm surfaces the structured Failure.
    //
    // Arc 170 slice 4c-α-ii: migrated from `:wat::kernel::run-sandboxed-ast`
    // to `:wat::test::run-thread`. The body does not read stdio slots,
    // does not call stdio verbs, and does not mutate runtime config —
    // FM 7-ter's three rules do not fire, so thread is the correct
    // (cheaper) destination. The outer only inspects `RunResult/failure`
    // which the thread driver populates from the cascade chain.
    match run_fn(":my::compute-assertion-failure") {
        Value::i64(n) => assert_eq!(n, 1, "expected failure to be detected (1); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

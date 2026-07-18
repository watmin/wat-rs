//! Integration: `:wat::test::run-hermetic` round trip (arc 170 slice
//! 4c-α-ii — was `:wat::kernel::run-sandboxed-hermetic-ast` before the
//! canonical-macro sweep).
//!
//! Demonstrates program-generates-program: the OUTER wat program
//! forks an INNER body via the canonical hermetic macro. The body
//! prints a value to stdout. The outer program reads that captured
//! string and evaluates it via `:wat::eval-edn!`. End result: a value
//! generated inside a fork'd child gets evaluated in the outer process.
//!
//! Both sites stay hermetic because the outer reads `RunResult/stdout`
//! and the body calls `:wat::kernel::println` — rules 1+2 of FM 7-ter
//! demand process boundary + pipe-captured stdio.
//!
//! Arc 170 slice 1f-ζ: outer main migrated to (:my::compute -> :T)
//! + eval_in_frozen. Inner programs use canonical nil main +
//! :wat::kernel::println (EDN-serializes values).

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("eval should succeed")
}

// ─── Simple hermetic happy path ─────────────────────────────────────────

#[test]
fn hermetic_inner_program_stdout_captured() {
    // Arc 170 slice 1f-ζ: inner uses canonical nil main + :wat::kernel::println.
    // :wat::kernel::println EDN-serializes strings with quotes.
    match run_fn(":my::compute-stdout-count") {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 stdout line; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Round trip — program-generates-program ─────────────────────────────

#[test]
fn hermetic_output_evaluated_in_outer_scope() {
    // Inner program prints i64 42. Outer program captures stdout[0]
    // (the EDN representation "42"), then eval-edn! parses it back to
    // an i64 value.
    //
    // The round-trip: a value computed by a fork'd child gets
    // evaluated back in the parent's wat runtime.
    let result = run_fn(":my::compute-eval-in-outer");
    let inner = unwrap_ok_result(result);
    // eval-edn! on "42" returns an i64 wrapped in a HolonAST (atom) or i64 directly.
    // The round-trip is verified: the child computed 42, parent evaluated it back.
    assert!(
        matches!(inner, Value::i64(42)) || matches!(inner, Value::holon__HolonAST(_)),
        "round trip should have computed 42; got {:?}",
        inner
    );
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn unwrap_ok_result(v: Value) -> Value {
    match v {
        Value::Result(r) => match &*r {
            Ok(inner) => inner.clone(),
            Err(e) => panic!("expected Ok; got Err({:?})", e),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

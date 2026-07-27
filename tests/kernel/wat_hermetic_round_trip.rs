//! Integration: hermetic round trip over the PRIMED peer wire (arc 278
//! IPC de-prime — was `:wat::test::run-hermetic` + `RunResult/stdout`, an
//! OS-pipe stdout scrape; migrated onto `spawn-program'` + `recv'`, the
//! outcome-walled peer primitive that `run-hermetic'` already rides).
//!
//! Demonstrates program-generates-program: the OUTER wat program forks an
//! INNER `:user::main` via `(:wat::spawn::process)`. The child body
//! `(:wat::kernel::println <value>)`s a value; on the primed wire that
//! value crosses to the parent as a decoded MESSAGE. The parent
//! `(:wat::kernel::recv p)` and matches `RecvOutcome::Message[m]` — `m`
//! IS the child's emitted value, which is exactly what the old stdout
//! scrape captured. `Lost`/`Closed` raise via `assertion-failed!`.
//!
//! End result: a value generated inside a fork'd child is captured + used
//! in the outer process — now as a first-class value over the peer channel,
//! not a scraped stdout string.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

// ─── Simple hermetic happy path ─────────────────────────────────────────

#[test]
fn hermetic_inner_program_stdout_captured() {
    // Inner program prints one value; the parent recv's exactly one
    // RecvOutcome::Message → the fn returns 1 (the captured-value count).
    match run_fn(":my::compute-stdout-count") {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 captured message; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Round trip — program-generates-program ─────────────────────────────

#[test]
fn hermetic_output_evaluated_in_outer_scope() {
    // Inner program prints i64 42. The peer wire delivers it to the parent
    // as a decoded RecvOutcome::Message[m], so `m` is the native i64 42 —
    // the parent uses the value the child computed.
    //
    // The round-trip: a value computed by a fork'd child is captured back
    // in the parent's wat runtime as a first-class value.
    match run_fn(":my::compute-eval-in-outer") {
        Value::i64(n) => assert_eq!(n, 42, "round trip should have captured 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

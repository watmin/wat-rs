//! Arc 259 S2d — the RAII hinge: a peer reaps on scope-exit WITHOUT `close'`.
//!
//! S2d retires user-facing `close'` (the converged model: teardown is RAII Drop;
//! the user never holds the rope). That is only honest if simply letting a peer's
//! binding leave scope DRAINS (drops the input sender → the worker's cascade-aware
//! `recv'` raises) THEN joins — no hang. S2b shipped that `Drop` on the Rust
//! `Thread`; this verifies it holds for a peer used from WAT and dropped with NO
//! `close'` call anywhere.
//!
//! HINGE-VERIFICATION probe (green ⇒ `close'` can retire), NOT a RED-at-HEAD
//! disconfirmer — S2b's RAII already ships; S2d builds on it. If the hinge were
//! broken (Drop joined a still-blocked worker WITHOUT draining first), the second
//! test would HANG.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_s2d_raii_hinge`
//!
//! WAT fixtures: tests/kernel/probe_arc259_s2d_raii_hinge_{used,blocked}.wat

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_compute_i64(path: &str) -> i64 {
    let world = startup_from_file(path).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// A peer used (send → echo → recv) then DROPPED without `close'`: the let-binding
/// leaves scope, RAII reaps the (already-exited) worker, the program completes.
/// NOTE: there is NO `close'` call — the worker is reaped by Drop alone.
#[test]
fn peer_used_then_dropped_without_close() {
    let got = run_compute_i64("tests/kernel/probe_arc259_s2d_raii_hinge_used.wat");
    assert_eq!(got, 99, "a peer used then dropped (no close') round-trips + reaps");
}

/// THE HINGE: a peer BLOCKED on `recv'` (never sent to, never `close'`d) is dropped
/// at scope-exit. RAII must DRAIN before join (drop the input sender → the worker's
/// `recv'` raises → the worker exits → join completes), so the program does NOT
/// hang. join-without-drain would deadlock here; this test completing IS the proof.
#[test]
fn blocked_peer_dropped_without_close_does_not_hang() {
    let got = run_compute_i64("tests/kernel/probe_arc259_s2d_raii_hinge_blocked.wat");
    assert_eq!(got, 7, "dropping a recv'-blocked peer (no close') must not hang");
}

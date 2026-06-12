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
//!   `cargo test --release -p wat --test nursery probe_arc259_s2d_raii_hinge -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute_i64(decls: &str) -> i64 {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
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
    let got = run_compute_i64(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::let \
             [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                     (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                       (:wat::kernel::send' self (:wat::kernel::recv' self)))) \
              _ (:wat::kernel::send' peer 99) \
              got (:wat::kernel::recv' peer)] \
             got))",
    );
    assert_eq!(got, 99, "a peer used then dropped (no close') round-trips + reaps");
}

/// THE HINGE: a peer BLOCKED on `recv'` (never sent to, never `close'`d) is dropped
/// at scope-exit. RAII must DRAIN before join (drop the input sender → the worker's
/// `recv'` raises → the worker exits → join completes), so the program does NOT
/// hang. join-without-drain would deadlock here; this test completing IS the proof.
#[test]
fn blocked_peer_dropped_without_close_does_not_hang() {
    let got = run_compute_i64(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::do \
             (:wat::core::let \
               [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                       (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                         (:wat::kernel::send' self (:wat::kernel::recv' self))))] \
               nil) \
             7))",
    );
    assert_eq!(got, 7, "dropping a recv'-blocked peer (no close') must not hang");
}

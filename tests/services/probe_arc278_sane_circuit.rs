//! The sane circuit — consumers that consume, a shutdown that is a signal.
//! Companion to `wat-scripts/fanout/circuit.wat`. The floor-weight lossless
//! summary stays on `probe_ex001_fanout.rs` (`:user::compute`). These gates
//! prove the contract the old `(range 0 cap)` worker could not: in-flight is
//! load-bearing, Stop is prompt, empty polls are gone.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_circuit() -> FrozenWorld {
    let rel = "wat-scripts/fanout/circuit.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("sane circuit should freeze")
}

fn call_string(world: &FrozenWorld, name: &str) -> String {
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{name} returned non-String: {other:?}"),
        Err(e) => panic!("{name} raised: {e:?}"),
    }
}

fn field<'a>(summary: &'a str, key: &str) -> &'a str {
    for part in summary.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("summary missing field {key:?}: {summary}");
}

/// Row 2: drain on pending=0 alone, with a delayed-ack worker, MUST lose the
/// held message. If this still passes, the in-flight term is not doing the work.
#[test]
fn pending_only_drain_loses_in_flight() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::pending-only-loses");
    assert_eq!(
        field(&stored, "lost"),
        "yes",
        "pending-only drain must lose the unacked message (distinct < n); got {stored}"
    );
    let n: i64 = field(&stored, "n")
        .parse()
        .unwrap_or_else(|_| panic!("n not an i64 in {stored}"));
    let distinct: i64 = field(&stored, "distinct")
        .parse()
        .unwrap_or_else(|_| panic!("distinct not an i64 in {stored}"));
    assert!(
        distinct < n,
        "in-flight term is load-bearing only if removing it fails: distinct={distinct} n={n} in {stored}"
    );
}

/// Row 5: Admin::Stop while a worker is ticking on an empty queue returns
/// promptly (the tick returns to the serve loop; Stop is taken there).
#[test]
fn stop_while_idle_is_prompt() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::stop-idle");
    let dt: i64 = field(&stored, "dt-ms")
        .parse()
        .unwrap_or_else(|_| panic!("dt-ms not an i64 in {stored}"));
    assert!(
        dt < 500,
        "Stop while ticking must return promptly; got dt-ms={dt} in {stored}"
    );
}

/// S13 row 1: a forced redelivery is visible as a message duplicate (same seq,
/// different envelope ids). Dedupe is off — the parent records both receives.
#[test]
fn redelivery_is_visible_as_a_message_duplicate() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::redelivery-is-visible");
    assert_eq!(
        field(&stored, "same-seq"),
        "yes",
        "redelivery must be the same published seq; got {stored}"
    );
    assert_eq!(
        field(&stored, "envelopes-differ"),
        "yes",
        "a redelivery is a new envelope; if ids match the detector is still blind; got {stored}"
    );
    assert_eq!(
        field(&stored, "distinct"),
        "1",
        "distinct on seq must stay 1 while total rises; got {stored}"
    );
    let total: i64 = field(&stored, "total")
        .parse()
        .unwrap_or_else(|_| panic!("total not an i64 in {stored}"));
    let dup: i64 = field(&stored, "dup")
        .parse()
        .unwrap_or_else(|_| panic!("dup not an i64 in {stored}"));
    assert!(total > 1, "forced redelivery must produce two receives; got {stored}");
    assert!(dup > 0, "dup must be visible; got {stored}");
}

/// S13 row 2: the same redelivery, absorbed by the consumer. One outcome.
#[test]
fn redelivery_is_absorbed_by_the_consumer() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::redelivery-is-absorbed");
    assert_eq!(
        field(&stored, "total"),
        "1",
        "an idempotent consumer must emit one outcome for a redelivered message; got {stored}"
    );
    assert_eq!(field(&stored, "distinct"), "1", "got {stored}");
    assert_eq!(field(&stored, "dup"), "0", "got {stored}");
    let seen_dups: i64 = field(&stored, "seen-dups")
        .parse()
        .unwrap_or_else(|_| panic!("seen-dups not an i64 in {stored}"));
    assert!(
        seen_dups > 0,
        "the ledger must count the absorbed redelivery; a counter that never counts is a deleted counter; got {stored}"
    );
}

/// Row 7: receive-calls approach the message count, not ~3× it. Floor weight
/// is 12×2 = 24 messages; the old worker did 4×12 = 48 polls by construction.
#[test]
fn receive_calls_are_not_triple_the_messages() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::compute-calls");
    let calls: i64 = field(&stored, "calls")
        .parse()
        .unwrap_or_else(|_| panic!("calls not an i64 in {stored}"));
    assert!(calls > 0, "expected some receive calls; got {stored}");
    assert!(
        calls < 72,
        "receive-calls must approach the message count (24), not ~3× it (72); got {stored}"
    );
}

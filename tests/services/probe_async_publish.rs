//! Durable topic — accepted means the N inbox rows exist. Drives gates in
//! `wat-scripts/topic/sns-fanout.wat` and the circuit's inbox-term row.
//!
//! `startup_from_file` uses `InMemoryLoader` and cannot resolve this file's
//! relative `load-file!` of `../queue/sqs.wat`. Drive it the way
//! `outbox_term_removed_loses_messages` does: `startup_from_source` + `FsLoader`.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_topic() -> FrozenWorld {
    let rel = "wat-scripts/topic/sns-fanout.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader)).expect("topic should freeze")
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

#[test]
fn publish_ok_means_durable() {
    let world = load_topic();
    let stored = call_string(&world, ":user::durable-ok");
    assert_eq!(
        field(&stored, "durable"),
        "yes",
        "publish then read inbox depth before any delivery: the message is in the store; got {stored}"
    );
    let n: i64 = field(&stored, "pending")
        .parse()
        .unwrap_or_else(|_| panic!("pending not an i64 in {stored}"));
    assert!(
        n >= 1,
        "inbox depth after one publish with no workers must be >= 1; got {stored}"
    );
}

#[test]
fn unit_is_per_subscription() {
    let world = load_topic();
    let stored = call_string(&world, ":user::unit-is-per-sub");
    assert_eq!(
        field(&stored, "unit"),
        "per-sub",
        "one publish to N=3 must write 3 rows, not 1; got {stored}"
    );
    assert_eq!(field(&stored, "rows"), "3");
}

#[test]
fn refused_subscriber_is_retried_not_dropped() {
    let world = load_topic();
    let stored = call_string(&world, ":user::refused-is-retried");
    assert_eq!(
        field(&stored, "inflight"),
        "yes",
        "worker must hold the row unacked while the subscriber is full; got {stored}"
    );
    assert_eq!(
        field(&stored, "after-drain"),
        "none",
        "after the dummy is gone the real message must still be invisible (in-flight); got {stored}"
    );
    assert_eq!(
        field(&stored, "after-expiry"),
        "got",
        "visibility expiry must deliver to the now-free subscriber; got {stored}"
    );
}

#[test]
fn stalled_subscriber_does_not_stall_others() {
    let world = load_topic();
    let stored = call_string(&world, ":user::stalled-does-not-stall");
    assert_eq!(
        field(&stored, "healthy"),
        "got",
        "the free subscriber must receive immediately; got {stored}"
    );
    assert_eq!(
        field(&stored, "blocked"),
        "no",
        "publish must not wait on the stalled subscriber; got {stored}"
    );
}

#[test]
fn publish_returns_before_delivery() {
    let world = load_topic();
    let stored = call_string(&world, ":user::publish-is-async");
    assert_eq!(
        field(&stored, "prompt"),
        "yes",
        "publish must return after the inbox write, with no workers running; got {stored}"
    );
}

#[test]
fn full_inbox_refuses_not_drops() {
    let world = load_topic();
    let stored = call_string(&world, ":user::inbox-refuses");
    assert_eq!(
        stored, "a=ok;b=ok;c=full",
        "cap 2: third publish is Full, not a silent drop. got: {stored}"
    );
}

/// Row 2 of D2: force the publish liveness bound to expire. A bound that only
/// says "gave up" fails — it must name depth, cap, attempts, elapsed.
#[test]
fn publish_liveness_bound_reports_what_it_saw() {
    let world = load_topic();
    let stored = call_string(&world, ":user::publish-bound-reports");
    assert_eq!(
        field(&stored, "verdict"),
        "never-accepted",
        "limit-ms 0 against a full inbox must trip the bound; got {stored}"
    );
    assert_eq!(
        field(&stored, "depth"),
        "2",
        "Full must report the depth it saw; got {stored}"
    );
    assert_eq!(
        field(&stored, "cap"),
        "2",
        "Full must report the cap it saw; got {stored}"
    );
    let attempts: i64 = field(&stored, "attempts")
        .parse()
        .unwrap_or_else(|_| panic!("attempts not an i64 in {stored}"));
    assert!(
        attempts >= 1,
        "the bound must count the try that saw Full; got {stored}"
    );
    let elapsed: i64 = field(&stored, "elapsed")
        .parse()
        .unwrap_or_else(|_| panic!("elapsed not an i64 in {stored}"));
    assert!(
        elapsed >= 0,
        "the bound must report elapsed; got {stored}"
    );
}

#[test]
fn idle_topic_never_ticks() {
    let world = load_topic();
    let stored = call_string(&world, ":user::idle-ticks");
    assert_eq!(
        stored, "ticks=0",
        "an idle topic must not tick; got {stored}"
    );
}

#[test]
fn outbox_term_removed_loses_messages() {
    let rel = "wat-scripts/fanout/circuit.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let world = startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("circuit should freeze");
    let stored = call_string(&world, ":user::outbox-term-loses");
    assert_eq!(
        field(&stored, "lost"),
        "yes",
        "drain without the inbox term must lose accepted-but-undelivered messages; got {stored}"
    );
    let n: i64 = field(&stored, "n")
        .parse()
        .unwrap_or_else(|_| panic!("n not an i64 in {stored}"));
    let distinct: i64 = field(&stored, "distinct")
        .parse()
        .unwrap_or_else(|_| panic!("distinct not an i64 in {stored}"));
    assert!(
        distinct < n,
        "inbox term is load-bearing only if removing it fails: distinct={distinct} n={n} in {stored}"
    );
}

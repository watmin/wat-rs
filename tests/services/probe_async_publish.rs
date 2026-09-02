//! Async topic publish — accepted, then fan-out. Drives gates in
//! `wat-scripts/topic/sns-fanout.wat` and the circuit's outbox-term row.

use std::sync::Arc;
use wat::freeze::{startup_from_file, startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

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
fn publish_returns_before_slow_subscriber() {
    let world = startup_from_file("wat-scripts/topic/sns-fanout.wat")
        .expect("topic should freeze");
    let stored = call_string(&world, ":user::publish-is-async");
    assert_eq!(
        field(&stored, "prompt"),
        "yes",
        "publish must return before a 200ms subscriber finishes; got {stored}"
    );
}

#[test]
fn full_outbox_refuses_not_drops() {
    let world = startup_from_file("wat-scripts/topic/sns-fanout.wat")
        .expect("topic should freeze");
    let stored = call_string(&world, ":user::outbox-refuses");
    assert_eq!(
        stored, "a=ok;b=ok;c=full",
        "cap 2: third publish is Full, not a silent drop. got: {stored}"
    );
}

#[test]
fn fanout_is_max_not_sum() {
    let world = startup_from_file("wat-scripts/topic/sns-fanout.wat")
        .expect("topic should freeze");
    let stored = call_string(&world, ":user::fanout-is-max");
    assert_eq!(
        field(&stored, "shape"),
        "max",
        "four 200ms subscribers must complete in ~max not ~sum; got {stored}"
    );
    let dt: i64 = field(&stored, "dt-ms")
        .parse()
        .unwrap_or_else(|_| panic!("dt-ms not an i64 in {stored}"));
    assert!(
        dt < 500,
        "concurrent fan-out is ~200ms, sequential is ~800ms; got dt-ms={dt} in {stored}"
    );
}

#[test]
fn idle_topic_never_ticks() {
    let world = startup_from_file("wat-scripts/topic/sns-fanout.wat")
        .expect("topic should freeze");
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
        "drain without the outbox term must lose accepted-but-undelivered messages; got {stored}"
    );
    let n: i64 = field(&stored, "n")
        .parse()
        .unwrap_or_else(|_| panic!("n not an i64 in {stored}"));
    let distinct: i64 = field(&stored, "distinct")
        .parse()
        .unwrap_or_else(|_| panic!("distinct not an i64 in {stored}"));
    assert!(
        distinct < n,
        "outbox term is load-bearing only if removing it fails: distinct={distinct} n={n} in {stored}"
    );
}

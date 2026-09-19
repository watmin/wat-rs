//! a-peer-that-is-merely-slow — floor harness for delay-bp + delay-ms
//! on `wat-scripts/query/faulting-store.wat`.
//!
//! The probe is `wat-scripts/scratch-pad/probe-a-peer-that-is-merely-slow.wat`.
//! Drive it the way `probe_store_injector_has_teeth.rs` does (FsLoader).
//!
//!   `:slow::run-rates`          delay-bp 0 vs 10000, delay-ms 200
//!   `:slow::run-late-vs-absent` recv-by-deadline fired by lateness, not absence
//!   `:slow::run-desync`         re-ask vs re-recv on a slow store reply
//!
//! Timing assertions are "at least N". Do not pin delays-fired to an exact count
//! beyond the disarmed zero. Knob defaults OFF: disarmed delays-fired must be 0.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_probe() -> FrozenWorld {
    let rel = "wat-scripts/scratch-pad/probe-a-peer-that-is-merely-slow.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("slow-peer probe should freeze")
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
            let k = k.rsplit(' ').next().unwrap_or(k);
            if k == key {
                return v;
            }
        }
    }
    panic!("summary missing field {key:?}: {summary}");
}

fn i64_field(summary: &str, key: &str) -> i64 {
    field(summary, key)
        .parse()
        .unwrap_or_else(|_| panic!("{key} not an i64 in {summary}"))
}

fn half(summary: &str, which: usize) -> &str {
    let parts: Vec<&str> = summary.split(" ;; ").collect();
    parts
        .get(which)
        .copied()
        .unwrap_or_else(|| panic!("summary missing half {which}: {summary}"))
}

#[test]
fn slow_peer_rates_differ_at_the_delay_site() {
    let world = load_probe();
    let stored = call_string(&world, ":slow::run-rates");
    let disarmed = half(&stored, 0);
    let armed = half(&stored, 1);
    eprintln!("rates {stored}");
    let off_fired = i64_field(disarmed, "delays-fired");
    let on_fired = i64_field(armed, "delays-fired");
    let on_elapsed = i64_field(armed, "elapsed-ms");
    assert_eq!(
        field(disarmed, "put"),
        "Success",
        "disarmed put must succeed; got {stored}"
    );
    assert_eq!(
        field(armed, "put"),
        "Success",
        "armed put must succeed after the wait; got {stored}"
    );
    assert_eq!(
        off_fired, 0,
        "disarmed delay-bp=0 must not fire; got {stored}"
    );
    assert!(
        on_fired > 0,
        "armed delay-bp=10000 must fire at the wait site; got {stored}"
    );
    assert!(
        on_elapsed >= 200,
        "armed delay-ms=200 must take at least 200ms; got {stored}"
    );
}

#[test]
fn slow_peer_deadline_fires_on_lateness_not_absence() {
    let world = load_probe();
    let stored = call_string(&world, ":slow::run-late-vs-absent");
    let late = half(&stored, 0);
    let absent = half(&stored, 1);
    eprintln!("late-vs-absent {stored}");
    assert_eq!(field(late, "sent"), "Sent", "late send; got {stored}");
    assert_eq!(field(absent, "sent"), "Sent", "absent send; got {stored}");
    assert_eq!(
        field(late, "first"),
        "TimedOut",
        "recv-by-deadline 50ms must fire against a 400ms peer; got {stored}"
    );
    assert_eq!(
        field(absent, "first"),
        "TimedOut",
        "recv-by-deadline 50ms must fire against a dropped reply; got {stored}"
    );
    assert_eq!(
        field(late, "second"),
        "Message",
        "a late reply must still arrive on the second recv-by-deadline; got {stored}"
    );
    assert_eq!(
        field(absent, "second"),
        "TimedOut",
        "an absent reply must still be absent on the second recv-by-deadline; got {stored}"
    );
    assert!(
        i64_field(late, "delays-fired") > 0,
        "late path must have waited; got {stored}"
    );
    assert_eq!(
        i64_field(absent, "delays-fired"),
        0,
        "drop-reply path must not increment delays-fired; got {stored}"
    );
}

#[test]
fn slow_peer_reask_leaves_a_surplus_rerecv_does_not() {
    let world = load_probe();
    let stored = call_string(&world, ":slow::run-desync");
    let reask = half(&stored, 0);
    let rerecv = half(&stored, 1);
    eprintln!("desync {stored}");
    assert_eq!(field(reask, "sent1"), "Sent", "reask sent1; got {stored}");
    assert_eq!(
        field(reask, "first"),
        "TimedOut",
        "reask first recv-by-deadline 50ms; got {stored}"
    );
    assert_eq!(field(reask, "sent2"), "Sent", "reask sent2; got {stored}");
    assert_eq!(
        field(reask, "after-reask"),
        "Message",
        "re-ask reads the abandoned first reply; got {stored}"
    );
    assert_eq!(
        field(reask, "leftover"),
        "Message",
        "re-ask leaves the second reply on the peer; got {stored}"
    );
    assert_eq!(field(rerecv, "sent1"), "Sent", "rerecv sent1; got {stored}");
    assert_eq!(
        field(rerecv, "first"),
        "TimedOut",
        "rerecv first recv-by-deadline 50ms; got {stored}"
    );
    assert_eq!(
        field(rerecv, "drained"),
        "Message",
        "re-recv drains the late first reply; got {stored}"
    );
    assert_eq!(field(rerecv, "sent2"), "Sent", "rerecv sent2; got {stored}");
    assert_eq!(
        field(rerecv, "own"),
        "Message",
        "after a drain the next send gets its own reply; got {stored}"
    );
    assert_eq!(
        field(rerecv, "leftover"),
        "TimedOut",
        "re-recv leaves nothing on the peer; got {stored}"
    );
}

//! the-bracket-runs-on-the-queue — control by mutation.
//!
//! Queue path: `bracket/map` over `queue-opts` with drop-recv-bp armed, asserting BOTH the result
//! and `recv-drops>0` so a pass cannot be a vacuous skip.
//!
//! ⛔ The "peer disposition" half is GONE — see the tombstone at the foot of this file. It matched
//! a panic string the fixture itself wrote, so it could not redden on the change it claimed to
//! control. Reachability is pinned by `probe_bare_recv_outcome_surface.rs` instead.

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn run_vec(path: &str) -> Vec<i64> {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::Vec(v) => v
            .iter()
            .map(|tv| match tv {
                Value::i64(n) => *n,
                other => panic!("non-i64 element: {other:?}"),
            })
            .collect(),
        other => panic!("expected Vector; got {other:?}"),
    }
}

fn run_string(path: &str) -> String {
    let world = startup_from_file(path).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
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

fn i64_field(summary: &str, key: &str) -> i64 {
    field(summary, key)
        .parse()
        .unwrap_or_else(|_| panic!("{key} not an i64 in {summary}"))
}

#[test]
fn queue_path_map_doubles_in_order() {
    let got = run_vec("tests/kernel/probe_bracket_runs_on_the_queue_map.wat");
    let expected: Vec<i64> = (1..=20).map(|n| n * 2).collect();
    assert_eq!(got, expected);
}

#[test]
fn throughput_queue_path_does_not_serialise() {
    let expected: Vec<i64> = (1..=20).map(|n| n * 2).collect();
    let t0 = std::time::Instant::now();
    let peer = run_vec("tests/kernel/probe_bracket_runs_on_the_queue_peer_map.wat");
    let peer_ms = t0.elapsed().as_millis();
    let t1 = std::time::Instant::now();
    let queue = run_vec("tests/kernel/probe_bracket_runs_on_the_queue_map.wat");
    let queue_ms = t1.elapsed().as_millis();
    assert_eq!(peer, expected);
    assert_eq!(queue, expected);
    eprintln!("throughput peer-ms={peer_ms} queue-ms={queue_ms}");
    // Serialising 20 items on one worker would still finish quickly; the
    // construction is N spawn-thread pullers. This bound is "did not hang
    // behind a single-file drain" — 20× the peer path would be a red flag.
    assert!(
        queue_ms < peer_ms.saturating_mul(20).max(5_000),
        "queue path serialised or stalled: peer-ms={peer_ms} queue-ms={queue_ms}"
    );
}

#[test]
fn queue_path_survives_recv_drop() {
    let summary = run_string("tests/kernel/probe_bracket_runs_on_the_queue_green.wat");
    assert_eq!(
        field(&summary, "result"),
        "2,4,6,8,10,12,14,16,18,20",
        "queue path must return doubled items, got {summary}"
    );
    let drops = i64_field(&summary, "recv-drops");
    assert!(
        drops > 0,
        "induced fault must have fired (recv-drops>0); got {summary}"
    );
}

// ⛔ `peer_disposition_dies_on_timedout` WAS HERE AND IS DELETED, 2026-09-19. It is recorded
// rather than quietly dropped, because the reason is load-bearing.
//
// It read a fixture (`…_peer_dies.wat`, deleted with it) that NEVER RAN A BRACKET: it started a
// queue at `drop-recv-bp=10000`, called `call-by-deadline` directly, and hand-wrote the two panic
// strings into its own match arms. So its `#[should_panic]` was satisfied by an
// `assertion-failed!` the fixture author typed in — delete all ten arms from `wat/bracket.wat` and
// it stayed green. It controlled nothing.
//
// ⭐ And the control it was standing in for was never constructible, because the premise was
// false: bare `:wat::kernel::recv` CANNOT return `RecvOutcome::TimedOut` or `::Malformed`, so
// bracket's ten arms are exhaustiveness filler, not live crash paths. The honest control is
// `tests/kernel/probe_bare_recv_outcome_surface.rs`, which pins that reachability and reddens if
// anything makes those arms live.

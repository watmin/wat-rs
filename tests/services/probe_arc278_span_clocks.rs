//! arc 278 item (c) stone B — two independent cadences.
//!
//! Time arrives as I/O: gates bound on the observed store count via poll-until;
//! nap is select' on a one-shot after. Never sleep-then-assert.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn run(name: &str) -> Value {
    let world = startup_beside(file!()).unwrap_or_else(|e| panic!("startup failed: {e:?}"));
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("{name} raised: {e:?}"))
}

#[test]
fn only_logs_flushes_logs_and_zero_metrics() {
    let got = run(":user::only-logs");
    assert!(
        matches!(got, Value::i64(1)),
        "a span that only logs must flush logs and write zero metrics; got {got:?}"
    );
}

#[test]
fn only_counts_flushes_metrics_and_zero_logs() {
    let got = run(":user::only-counts");
    assert!(
        matches!(got, Value::i64(1)),
        "a span that only counts must flush metrics and write zero logs; got {got:?}"
    );
}

#[test]
fn tick_rearms_without_client_flush() {
    let got = run(":user::rearm");
    assert!(
        matches!(got, Value::i64(n) if n >= 2),
        "one span, no Span/flush, must observe ≥2 log flushes (arm then re-arm); got {got:?}"
    );
}

#[test]
fn idle_span_is_silent() {
    let got = run(":user::idle");
    assert!(
        matches!(got, Value::i64(1)),
        "a span that never logs or counts must write nothing over several intervals; got {got:?}"
    );
}

#[test]
fn non_default_cadence_is_honoured() {
    let got = run(":user::cadence");
    assert!(
        matches!(got, Value::i64(1)),
        "a 20ms logs cadence must flush before a 2000ms one; got {got:?}"
    );
}

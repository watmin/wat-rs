//! arc 278 item (c) stone D — the bounded buffer.
//!
//! logs and duration samples are bounded in items. Overflow drops the oldest,
//! increments :logs-dropped / :samples-dropped, and returns Dropped{buffered, cap}.

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
fn bound_holds() {
    let got = run(":user::bound-holds");
    assert!(
        matches!(got, Value::i64(1)),
        "logging past logs-max against a failing sink must leave the buffer at logs-max; got {got:?}"
    );
}

#[test]
fn drop_count_is_exact() {
    let got = run(":user::drop-count-exact");
    assert!(
        matches!(got, Value::i64(1)),
        ":logs-dropped must be exactly 7 after 10 logs with logs-max 3; got {got:?}"
    );
}

#[test]
fn overflowing_log_returns_dropped_not_ok() {
    let got = run(":user::caller-told");
    assert!(
        matches!(got, Value::i64(1)),
        "the log that overflows must return Dropped, never Ok; got {got:?}"
    );
}

#[test]
fn oldest_logs_are_dropped() {
    let got = run(":user::oldest-go");
    assert!(
        matches!(got, Value::i64(1)),
        "after overflow the buffer must hold the most recent logs-max, in order (7,8,9); got {got:?}"
    );
}

#[test]
fn samples_bound_and_oldest_and_dropped_response() {
    let got = run(":user::samples-bound");
    assert!(
        matches!(got, Value::i64(1)),
        "timed past duration-samples-max must keep the 3 newest samples and return Dropped; got {got:?}"
    );
}

#[test]
fn samples_drop_count_is_exact() {
    let got = run(":user::samples-drop-count");
    assert!(
        matches!(got, Value::i64(1)),
        ":samples-dropped must be exactly 7 after 10 timed with duration-samples-max 3; got {got:?}"
    );
}

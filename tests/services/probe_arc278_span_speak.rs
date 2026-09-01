//! arc 278 item (c) stone C — a size-triggered flush must speak.
//!
//! A failing sink driven past the journal write cap must surface Constraint/Transient/Fatal
//! on the op that triggered the flush, never Ok. The arriving item stays in the buffer.

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
fn logs_size_flush_failure_reaches_the_caller() {
    let got = run(":user::logs-speak");
    assert!(
        matches!(got, Value::i64(1)),
        "a size-triggered log flush against a failing sink must report Fatal (not Ok); got {got:?}"
    );
}

#[test]
fn logs_arriving_item_survives_failed_flush() {
    let got = run(":user::logs-survive");
    assert!(
        matches!(got, Value::i64(1)),
        "after a failed size-triggered flush the durable buffer must hold the refused batch AND the arriving log; got {got:?}"
    );
}

#[test]
fn timed_size_flush_failure_and_arriving_sample_survive() {
    let got = run(":user::timed-speak-survive");
    assert!(
        matches!(got, Value::i64(1)),
        "timed past the metrics cap against a failing sink must speak and keep every sample; got {got:?}"
    );
}

#[test]
fn incr_size_flush_failure_and_arriving_count_survive() {
    let got = run(":user::incr-speak-survive");
    assert!(
        matches!(got, Value::i64(1)),
        "incr past the metrics cap against a failing sink must speak and keep every counter; got {got:?}"
    );
}

#[test]
fn ok_still_means_accepted() {
    let got = run(":user::ok-accepted");
    assert!(
        matches!(got, Value::i64(1)),
        "a normal log with no size trigger must still be LogResponse::Ok; got {got:?}"
    );
}

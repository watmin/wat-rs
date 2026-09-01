//! arc 278 item (c) stone A — the buffered span.
//!
//! Double-count gate: incr ×3, flush, incr ×2, close → sum of :requests metrics is exactly 5.
//! Also: empty second flush emits nothing; duration count+sum unchanged AND one /sample per
//! sample; buffered logs all land on close.

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
fn incr_flush_incr_close_sums_to_exactly_five() {
    let got = run(":user::double-count");
    assert!(
        matches!(got, Value::i64(5)),
        "incr×3 → flush → incr×2 → close must sum :requests to exactly 5 (an 8 is totals-not-deltas); got {got:?}"
    );
}

#[test]
fn second_flush_and_empty_close_emit_nothing() {
    let got = run(":user::flush-empty");
    assert!(
        matches!(got, Value::i64(1)),
        "incr×1 → flush → flush → close must leave exactly 1 metric row (second flush and close emit nothing); got {got:?}"
    );
}

#[test]
fn duration_emits_count_sum_and_one_sample_per_sample() {
    let got = run(":user::duration-fidelity");
    assert!(
        matches!(got, Value::i64(5123)),
        "3 samples → 5 metrics (count+duration+3 samples); I64 sum 3+60+10+20+30=123 (packed 5123); got {got:?}"
    );
}

#[test]
fn buffered_logs_all_land_on_close() {
    let got = run(":user::logs-survive");
    assert!(
        matches!(got, Value::i64(5)),
        "five buffered logs must all land in the store on close, in the batch; got {got:?}"
    );
}

//! arc 278 STONE Span.2 — `span'` the PRODUCER service, end-to-end. A real span' (given a journal'
//! given a mem-store') accumulates a counter (`incr :requests` twice) and on `close` emits it as a
//! Metric to the sink; the sink persists it. A separate client scans the store, hydrates the one
//! Metric, and returns its counter value — which must be 2. Proves the producer->sink->store chain
//! (accumulate + emit-on-close), the meaty half of the Span facility.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn span_accumulates_a_counter_and_emits_it_as_a_metric_on_close() {
    let world = startup_beside(file!())
        .expect("startup should succeed (span' + journal' + mem-store' + telemetry vocab baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("span' incr/close -> journal' -> store round-trip raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(2)),
        "expected span' to accumulate :requests=2 and emit a Count Metric with value 2 through the \
         sink to the store; got {got:?} (-1 = wrong Numeric variant, -2 = wrong row count, -3 = scan failed)"
    );
}

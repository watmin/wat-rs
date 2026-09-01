//! arc 278 STONE Span.3 — the call-site macros `with-span` + `timed`. Inside one `with-span`
//! (inline open/use/close), incr :requests twice + `timed` a body once. The macro's close emits
//! FOUR Metrics: :requests (ONE aggregated counter = 2), :fetch/count, :fetch/duration, and one
//! :fetch/sample (item (c) stone A — fidelity). A client scans the store and the row count must be 4
//! — proving with-span opened + closed, timed fed Span/timed, incr aggregated (not one Metric per
//! incr), and close emits the sample.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn with_span_and_timed_emit_the_aggregated_metrics_on_close() {
    let world = startup_beside(file!())
        .expect("startup should succeed (span' + its macros + journal' + mem-store' baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("with-span / timed round-trip raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(4)),
        "expected with-span's close to emit 4 Metrics (1 aggregated :requests counter + \
         :fetch/count + :fetch/duration + 1 :fetch/sample); got {got:?} (a count != 4 means incr \
         fanned out, timed didn't record, close didn't fire, or samples were dropped)"
    );
}

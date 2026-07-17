//! arc 278 T2 — the CloudWatch READ side: journal' query-metrics. Write 2 Metrics (t=1s, t=2s),
//! then query-metrics back: a BROAD [0,3s] window returns both, a NARROW [1.5s,3s] window returns
//! one (time-range filtering drops the t=1s metric). Encoded as broad*10 + narrow = 21 — proving
//! read-back (scan + hydrate off the tag) and time-window filtering, no rete anywhere.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_query_metrics_reads_back_and_filters_by_time_window() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' with query ops + mem-store' baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("journal' query-metrics round-trip raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(21)),
        "expected broad query to return 2 metrics and narrow query to return 1 (encoded 2*10+1=21); \
         got {got:?} (a value like 2N means the narrow window's time filter failed to drop the t=1s metric)"
    );
}

//! arc 278 T2 — query-logs (thread). Closes the un-probed query-logs gap: write 2 Logs (t=1s,2s),
//! query-logs a BROAD [0,3s] window (2) + a NARROW [1.5s,3s] window (1). Encoded 2*10+1 = 21.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_query_logs_reads_back_and_filters_by_time_window() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("query-logs round-trip raised: {e:?}"));
    assert!(matches!(got, Value::i64(21)),
        "expected broad=2, narrow=1 (2*10+1=21); got {got:?}");
}

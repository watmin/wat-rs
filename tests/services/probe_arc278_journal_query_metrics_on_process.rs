//! arc 278 T2 loci parity — query-metrics across a PROCESS fork. journal' + mem-store' forked
//! (grant-before-dial); write 2 Metrics, query-metrics [0,3s]; the forked journal' scans+hydrates
//! in the child and the response crosses the wire back. Count must be 2.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_query_metrics_across_a_process_fork() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("query-metrics on a process fork raised: {e:?}"));
    assert!(matches!(got, Value::i64(2)),
        "expected query-metrics across a fork to return 2 metrics; got {got:?}");
}

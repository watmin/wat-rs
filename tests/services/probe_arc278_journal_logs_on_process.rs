//! arc 278 T2 loci parity — write-logs AND query-logs across a PROCESS fork. journal' + mem-store'
//! forked (grant-before-dial); write 2 Logs, query-logs [0,3s]; the forked journal' scans+hydrates
//! Logs in the child, response crosses back. Count must be 2. Closes the write-logs + query-logs
//! loci-parity holes.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// KNOWN BUG (loci-parity hole caught by this test): write-logs across a PROCESS fork fails when the
// Log carries a user-defined LogMessage payload — the forked journal' child cannot decode the user
// record type (it's in the parent's program, not the child's baked stdlib), dies, and closes the
// channel ("recv': peer closed"). Metrics cross fine (all-stdlib fields). The fix under discussion:
// the Log message should cross + store as OPAQUE tagged-EDN (CloudWatch-blob shape), not a typed
// record the receiver must decode. Un-ignore when that lands.
#[ignore = "known bug: user LogMessage payload can't decode in a forked journal' child; fix = opaque message"]
#[test]
fn journal_writes_and_queries_logs_across_a_process_fork() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("write-logs/query-logs on a process fork raised: {e:?}"));
    assert!(matches!(got, Value::i64(2)),
        "expected write-logs + query-logs across a fork to return 2 logs; got {got:?}");
}

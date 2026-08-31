//! Excursus 001 SORTKEY — the same-nanosecond sequence the journal census named and
//! stopped short of: three Metrics at one `time-ns`, distinct event-ids, both backends.
//! All three must survive. Pre-fix this stored 1 (last-wins).
//!
//! Shape copied from `probe_arc278_journal_backend_differential`: helper on store Address,
//! both stores started, run both, compare. `assert_eq!` of the whole summary — no
//! `.contains(` (no_loose_string_assert).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

const AGREED_SUMMARY: &str = "count=3;names=a,b,c";

#[test]
fn same_ns_three_metrics_survive_on_mem_and_sqlite() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' + sqlite-store' baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("same-ns differential raised: {e:?}"),
    };
    assert_eq!(
        stored, AGREED_SUMMARY,
        "three same-ns Metrics with distinct event-ids must all survive on both backends. \
         A DIFFERENTIAL-MISMATCH prefix means mem and sqlite diverged. got: {stored}"
    );
}

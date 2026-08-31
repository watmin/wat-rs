//! Excursus 001 SORTKEY STOP-2 — a row at exactly `time-hi` is returned by
//! `query-metrics`, and the all-f uuid sentinel is actually maximal.
//! Demonstrated, not argued. `assert_eq!` of the whole summary.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

const AGREED_SUMMARY: &str = "hi=2;wide=3;nil<=max=1;mid<=max=1;high<=max=1;next>max=1;helper=1";

#[test]
fn sortkey_hi_sentinel_includes_a_row_at_exactly_time_hi() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' + SortKey baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("sortkey boundary raised: {e:?}"),
    };
    assert_eq!(
        stored, AGREED_SUMMARY,
        "query-metrics [T,T] must return both rows at T (including a non-nil event-id); \
         all-f must be the lexicographic max at that Instant; sort-key-hi(T) must equal \
         write(SortKey T all-f). got: {stored}"
    );
}

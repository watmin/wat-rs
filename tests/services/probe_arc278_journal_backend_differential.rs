//! arc 278 STONE T1b.3 — the journal backend differential (thread). The store is a swappable CONFIG
//! PARAM: the SAME journal' is run over mem-store' (oracle) and sqlite-store' (:memory:, the real
//! backend), selected only by which store's Address' is injected. Same write-metrics -> the two
//! backends must persist BIT-FOR-BIT identical rows. journal' names only the :wat::query::Store
//! surface, so any future backend (mysql/mongo/dynamo/es/redis/wat-built) slots in the same way.
//!
//! The .wat asserts the differential (returns the row IFF mem == sqlite, else a sentinel); the .rs
//! confirms the backends agreed AND that the agreed row is the valid, expected Metric EDN.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_persists_identically_across_mem_and_sqlite_backends_on_a_thread() {
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
        Err(e) => panic!("journal backend differential raised: {e:?}"),
    };
    // The differential: mem and sqlite must persist the SAME row (the .wat returns the mem row's
    // data only when it byte-equals the sqlite row's data).
    assert_ne!(
        stored, "DIFFERENTIAL-MISMATCH",
        "journal' persisted DIFFERENT rows to mem-store' vs sqlite-store' — the backends diverged"
    );
    // And the agreed row is the expected, valid Metric EDN (same golden as the process tier).
    wat::assert_edn_matches_file!(stored, "probe_arc278_journal__stored_metric.edn", "journal' persists the Metric's tagged EDN identically across mem + sqlite (config-param backend)");
}

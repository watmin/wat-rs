//! arc 278 STONE T1b.2 acceptance gate — `:wat::telemetry'::journal'` (the telemetry sink) given a
//! `mem-store'`, `write-metrics` a 1-Metric batch, then a SEPARATE client scans the store back and
//! we golden-compare the stored `data`. This composes everything the T1b groundwork proved:
//!   - a defservice HOLDING a :peers Store peer (U1: probe_arc278_s2s_peer_on_{thread,process})
//!   - Metric -> tagged EDN in `data` (U2: probe_arc278_metric_edn_write)
//!   - tagged pk / #inst sk / #uuid gsi keys (probe_arc278_tagged_keys_store)
//!
//! GREEN means journal' assembles them correctly on a thread. The process-locus sibling
//! (probe_arc278_journal_service_on_process) proves the fork path.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_writes_a_metric_through_a_held_store_peer_on_a_thread() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' + telemetry vocab all baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("journal' write-metrics round-trip raised: {e:?}"),
    };
    // The stored `data` must be exactly the Metric's tagged EDN (journal' serialized it via
    // :wat::edn::write). assert_edn_eq! parses both sides, so this also confirms `data` is valid EDN.
    wat::assert_edn_eq!(
        stored,
        include_str!("probe_arc278_journal_service__stored_metric.edn"),
        "journal' stored the Metric's tagged EDN as the row's data (write path end-to-end)"
    );
}

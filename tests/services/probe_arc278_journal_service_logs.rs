//! arc 278 STONE T1b.2 — journal' write-LOGS coverage (symmetric to write-metrics). journal' given
//! a mem-store', write-logs a 1-Log batch whose message is OPAQUE (Stone B): the producer
//! `edn::write`s its payload record at the call site, so a String crosses + is stored verbatim.
//! A separate client scans back; golden-compare the stored `data`.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_writes_a_log_through_a_held_store_peer_on_a_thread() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' + telemetry vocab baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("journal' write-logs round-trip raised: {e:?}"),
    };
    // The stored `data` must be the Log's tagged EDN, with :message an OPAQUE String holding the
    // producer-`edn::write`n payload record's EDN text (Stone B). assert_edn_eq! parses both sides.
    wat::assert_edn_eq!(
        stored,
        include_str!("probe_arc278_journal_service_logs__stored_log.edn"),
        "journal' stored the Log's tagged EDN (write-logs path, opaque String message)"
    );
}

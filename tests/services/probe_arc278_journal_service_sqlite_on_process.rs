//! arc 278 STONE T1b.3 / U3 — the REAL backend (sqlite) on a PROCESS fork. journal' + sqlite-store'
//! both forked; sqlite-store' opens its own Connection in the child; journal' dials it via
//! grant-before-dial. Same golden as the mem-on-process + thread-differential tiers, so sqlite ≡ mem
//! on a fork. This closes U3 (sqlite-store' was previously thread-only) — journal' is now proven
//! backend-agnostic AND loci-agnostic across the real backend.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_writes_a_metric_through_a_held_sqlite_store_peer_on_a_process() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + sqlite-store' baked; each child re-bakes)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!(
            "journal' + sqlite across a PROCESS fork raised: {e:?}. A dial/timeout means \
             grant-before-dial failed; a 'no such table'/Fatal means the forked sqlite-store' child \
             did not open its own Connection + journal' :init did not ensure the schema in the child."
        ),
    };
    wat::assert_edn_matches_file!(stored, "probe_arc278_journal__stored_metric.edn", "journal' persisted the Metric's tagged EDN through a sqlite backend across a process fork (U3)");
}

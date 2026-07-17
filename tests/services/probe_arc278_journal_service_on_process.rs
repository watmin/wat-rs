//! arc 278 STONE T1b.2 loci parity — `journal'` write-metrics across a PROCESS FORK. Both
//! `mem-store'` and `journal'` fork to processes; journal' (a process child) dials mem-store'
//! (another process child) via grant-before-dial (post-spawn hook). Same golden as the thread
//! sibling — journal' must behave identically on both loci.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn journal_writes_a_metric_through_a_held_store_peer_on_a_process() {
    let world = startup_beside(file!())
        .expect("startup should succeed (journal' + mem-store' baked; each child re-bakes the same)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!(
            "journal' write-metrics across a PROCESS fork raised: {e:?}. A #wat.macro/ReservedPrefix \
             means the reserved-ns child re-declaration regressed; a dial/timeout means the \
             grant-before-dial ordering (journal's pid -> mem-store's gate) failed."
        ),
    };
    wat::assert_edn_eq!(
        stored,
        include_str!("probe_arc278_journal__stored_metric.edn"),
        "journal' stored the Metric's tagged EDN across a process fork (loci parity with the thread tier)"
    );
}

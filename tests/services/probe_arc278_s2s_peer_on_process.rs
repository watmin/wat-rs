//! arc 278 S4d — a `defservice` HOLDING a `:peers` peer to another service, RUN + ASSERTED across a
//! PROCESS FORK. This is the loci-parity half of `journal'`'s skeleton: `caller'` holds a client
//! `Peer'<Echo::Op,Echo::Reply>` in a ROOT `:ephemeral` field, declares `:peers [:probe::Echo]`, and
//! both services fork to PROCESSES with a `post-spawn` grant-before-dial hook.
//!
//! WHY THIS TEST HAS TO EXIST: the s2s peer-holding lived only in `wat-scripts/probes/arc-278/`,
//! which the wat-scripts load gate only TYPE-CHECKS — never RUN. The FORK path in particular
//! (the `:peers` manifest concat shipping `(:probe::Echo::surface-forms)` into caller''s child
//! bundle, plus the grant-before-dial ordering) was proven to compile and executed by nothing.
//! `journal'` process-hosted inherits this exact dance. Sibling `..._on_thread.rs` covers the
//! non-fork case.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn s2s_peer_holding_service_round_trips_on_a_process_locus() {
    let world =
        startup_beside(file!()).expect("startup should succeed (echo'/caller' are user-ns)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("s2s peer-holding on a process locus raised: {e:?}"));
    assert!(
        matches!(got, Value::String(ref s) if s.as_str() == "echo:hi"),
        "expected caller' (a defservice HOLDING a :peers Peer' to echo') to dial echo' across a \
         PROCESS FORK (grant-before-dial via post-spawn) and return \"echo:hi\"; got {got:?}. A \
         StartupError here means the :peers manifest concat did not ship Echo's surface-forms into \
         the forked caller child; a dial/timeout means the grant-before-dial ordering failed."
    );
}

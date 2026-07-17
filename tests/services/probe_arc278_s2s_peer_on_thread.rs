//! arc 278 S4d — a `defservice` HOLDING a `:peers` peer to another service, RUN + ASSERTED on a
//! THREAD locus. `journal'`'s exact skeleton: `caller'` holds a client `Peer'<Echo::Op,Echo::Reply>`
//! in a ROOT `:ephemeral` field, declares `:peers [:probe::Echo]`, and its `run` impl calls
//! `Echo/echo` through that held peer.
//!
//! WHY THIS TEST HAS TO EXIST: the s2s peer-holding lived only in `wat-scripts/probes/arc-278/`,
//! which `tests/lint/wat_scripts_fixes_load.rs` merely TYPE-CHECKS — it is never RUN. So the
//! mechanism `journal'` (and every future peer-holding service) sits on was proven to compile on
//! both loci and proven to execute on NEITHER. This promotes it to a run+assert system test.
//! Sibling `probe_arc278_s2s_peer_on_process.rs` covers the FORK (with grant-before-dial).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn s2s_peer_holding_service_round_trips_on_a_thread_locus() {
    let world = startup_beside(file!()).expect("startup should succeed (echo'/caller' are user-ns)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("s2s peer-holding on a thread locus raised: {e:?}"));
    assert!(
        matches!(got, Value::String(ref s) if s.as_str() == "echo:hi"),
        "expected caller' (a defservice HOLDING a :peers Peer' to echo') to dial echo' through the \
         held peer and return \"echo:hi\" on a THREAD locus; got {got:?}"
    );
}

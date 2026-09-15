//! a-peer-remembers-its-address — a dialed process-tier peer remembers;
//! an accepted peer reports None; the peek steals nothing; thread-tier is None.
//!
//! cargo nextest run --release -E 'test(a_peer_remembers_its_address)'

use wat::capability::is_capability_type_path;
use wat::freeze::call_beside_value;
use wat::kernel::spawn::{ADDRESS_TYPE_PATH, PEER_TYPE_PATH};
use wat::runtime::Value;

#[test]
fn a_peer_remembers_its_address() {
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!(":user::compute must eval; got {e:?}"));
    let s = match got {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    };
    assert_eq!(
        s.as_str(),
        "client-proc=Some;accepted-proc=None;fresh=Some;orig=10;fresh-reply=14;client-thread=None;accepted-thread=None;thread-orig=10;timer=None",
        "dialed process-tier is Some; accepted/thread/timer are None; both peers still serve after the peek; the remembered address dials a second connection"
    );
}

#[test]
fn a_peer_is_not_a_wire_type() {
    assert!(
        is_capability_type_path(ADDRESS_TYPE_PATH),
        "Address is portable — the control that the waist still lists it"
    );
    assert!(
        !is_capability_type_path(PEER_TYPE_PATH),
        "Peer must not become a wire type as a side effect of storing an Address"
    );
}

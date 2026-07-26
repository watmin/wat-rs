//! Arc 209 locus-parity foundation (was "host-parity-4a") — the `:wat::spawn::Locus` protocol.
//!
//! 4a-i and 4a-ii are coupled: `listener'` accepting an abstract `:wat::spawn::Locus` requires that
//! protocol to EXIST, and a protocol's method set is atomic at `defprotocol`. So this stone ships the
//! whole foundation:
//!   - `defprotocol :wat::spawn::Locus` with `launch<S,R,St>` + `extend-type :wat::spawn::ThreadOpts`
//!     (the deftest' capture strategy; serve invoked by keyword via `:wat::core::apply`) — wat/spawn.wat.
//!   - `infer_listener_prime` accepts an abstract `:wat::spawn::Locus` locus (check.rs) so a locus-blind
//!     `(listener' locus :S :R)` type-checks to `Bound<S,R>`; runtime dispatches on the concrete value.
//!
//! THIS PROBE proves the WIRING with a locus-agnostic `start-it [h <- :Locus]` that calls BOTH
//! locus-routed primitives: `(listener' h :i64 :i64)` (4a-i) and `(:wat::spawn::Locus/launch h …)`
//! (4a-ii), invoked with a concrete `(thread)`. A trivial serve (returns nil) proves the launch
//! plumbing — listener' accepts :Locus, launch dispatches to the ThreadOpts impl, apply invokes serve
//! by keyword, spawn-program' runs it on a fresh peer, and a `:wat::spawn::Spawned` handle comes back.
//! (The full poll round-trip is proven via defservice in 4a-iii's forward probe.)
//!
//! RED at HEAD (pre-foundation): `:wat::spawn::Locus` does not exist AND `listener'` rejects an abstract
//! locus. GREEN once the protocol + ThreadOpts impl + the listener' arm ship.
//!
//! Run: cargo test --release -p wat --test probe_arc209_locus_protocol_foundation

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn locus_protocol_foundation_listener_and_launch_route_through_abstract_locus() {
    // Wat source lives in the co-located fixture: probe_arc209_locus_protocol_foundation.wat
    let got = call_beside_value(file!(), ":user::go")
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: locus-agnostic start-it routed listener'(:Locus) + Locus/launch through (thread); got {got:?}"
    );
}

//! Arc 209 host-parity-4a (foundation: 4a-i + 4a-ii fused) — the `:wat::spawn::Host` protocol.
//!
//! 4a-i and 4a-ii are coupled: `listener'` accepting an abstract `:wat::spawn::Host` requires that
//! protocol to EXIST, and a protocol's method set is atomic at `defprotocol`. So this stone ships the
//! whole foundation:
//!   - `defprotocol :wat::spawn::Host` with `launch<S,R,St>` + `extend-type :wat::spawn::ThreadOpts`
//!     (the deftest' capture strategy; serve invoked by keyword via `:wat::core::apply`) — wat/spawn.wat.
//!   - `infer_listener_prime` accepts an abstract `:wat::spawn::Host` host (check.rs) so a host-blind
//!     `(listener' host :S :R)` type-checks to `Bound<S,R>`; runtime dispatches on the concrete value.
//!
//! THIS PROBE proves the WIRING with a host-agnostic `start-it [h <- :Host]` that calls BOTH
//! host-routed primitives: `(listener' h :i64 :i64)` (4a-i) and `(:wat::spawn::Host/launch h …)`
//! (4a-ii), invoked with a concrete `(thread)`. A trivial serve (returns nil) proves the launch
//! plumbing — listener' accepts :Host, launch dispatches to the ThreadOpts impl, apply invokes serve
//! by keyword, spawn-program' runs it on a fresh peer, and a `:wat::spawn::Spawned` handle comes back.
//! (The full poll round-trip is proven via defservice in 4a-iii's forward probe.)
//!
//! RED at HEAD (pre-foundation): `:wat::spawn::Host` does not exist AND `listener'` rejects an abstract
//! host. GREEN once the protocol + ThreadOpts impl + the listener' arm ship.
//!
//! Run: cargo test --release -p wat --test probe_arc209_host_protocol_foundation

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; A trivial serve loop — proves the launch WIRING (not the poll loop, which 4a-iii covers).
(:wat::core::defn :my::svc::serve
  [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
   l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>
   st      <- :wat::core::i64] -> :wat::core::nil
  nil)

;; Host-AGNOSTIC: the param is the abstract `:wat::spawn::Host`. Both host-routed primitives flow
;; through it — `listener'` (4a-i) and `Host/launch` (4a-ii).
(:wat::core::defn :user::start-it [h <- :wat::spawn::Host] -> :wat::spawn::Spawned
  (:wat::core::let
    [b  (:wat::kernel::listener' h :wat::core::i64 :wat::core::i64)
     l  (:wat::spawn::Bound/listener b)
     c0 (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>)
     sp (:wat::spawn::Host/launch h l c0 0 (:wat::core::keyword/from-string "my::svc::serve"))]
    sp))

;; Drive it with a concrete (thread): reaching `true` means the whole host-agnostic launch wired and
;; ran without crashing (listener' accepted :Host, launch dispatched, apply invoked serve, peer spawned).
(:wat::core::defn :user::go [] -> :wat::core::bool
  (:wat::core::let [h (:user::start-it (:wat::spawn::thread))]
    true))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn host_protocol_foundation_listener_and_launch_route_through_abstract_host() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Host protocol + ThreadOpts launch + listener' accepts :Host)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: host-agnostic start-it routed listener'(:Host) + Host/launch through (thread); got {got:?}"
    );
}

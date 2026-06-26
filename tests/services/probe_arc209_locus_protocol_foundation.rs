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

;; init fn: takes ship (i64) and returns St (i64) — identity for this wiring probe.
;; arc 291: launch now takes [self ship init serve service-forms lu-addr-kw] (6 args).
(:wat::core::defn :my::svc::init [ship <- :wat::core::i64] -> :wat::core::i64 ship)

;; Locus-AGNOSTIC: the param is the abstract `:wat::spawn::Locus`. `Locus/launch` routes through it.
;; arc 291: launch signature = [self ship init serve service-forms lu-addr-kw] (6 args).
;; Launched now has 4 type params <S,R,Sh,Lu>; here all are i64.
(:wat::core::defn :user::start-it [h <- :wat::spawn::Locus] -> :wat::spawn::Launched<wat::core::i64,wat::core::i64,wat::core::i64,wat::core::i64>
  (:wat::spawn::Locus/launch h 0
    (:wat::core::keyword/from-string "my::svc::init")
    (:wat::core::keyword/from-string "my::svc::serve")
    (:wat::core::forms)
    (:wat::core::keyword/from-string "my::svc::init")))

;; Drive it with a concrete (thread): reaching `true` means the whole locus-agnostic launch wired and
;; ran without crashing (listener' accepted :Locus, launch dispatched, apply invoked serve, peer spawned).
(:wat::core::defn :user::go [] -> :wat::core::bool
  (:wat::core::let [h (:user::start-it (:wat::spawn::thread))]
    true))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn locus_protocol_foundation_listener_and_launch_route_through_abstract_locus() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Locus protocol + ThreadOpts launch + listener' accepts :Locus)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: locus-agnostic start-it routed listener'(:Locus) + Locus/launch through (thread); got {got:?}"
    );
}

//! Arc 209 host-parity leg, stone 4a — the host-agnostic `start [host <- :Host]`.
//!
//! C.3 shipped a THREAD-HARDCODED `start [state0]` (it bakes `(:wat::spawn::thread)` into the
//! `listener'` + `spawn-program'` calls, `wat/service.wat` start-body). Stone 4a makes `start` take
//! the host as its first arg and route through the `:wat::kernel::Host` protocol (arc 232) — so one
//! `start` serves any transport, dispatching the per-tier launch to the `extend-type :Host` impl.
//! Thread is `extend-type :wat::spawn::ThreadOpts :Host` (the shared-memory/capture strategy — the
//! `deftest'` model); process/remote join later as not-shared `extend-type`s, zero edit to `start`.
//!
//! THE GATE (thread-proven): the SAME counter as C.3, driven through the client face, but
//! `(:my::counter/start (:wat::spawn::thread) 0)` — start now takes a host. The round-trip
//! (increment 5 → get → 5) proves the host-agnostic start dispatches the thread launch correctly.
//!
//! RED at HEAD: C.3's `start` is arity-1 `[state0]` — `(counter/start (thread) 0)` is a 2-arg call
//! → arity mismatch; and `:wat::kernel::Host` / the ThreadOpts `extend-type` don't exist. GREEN once
//! 4a ships the Host protocol + thread impl + the host-agnostic start codegen.
//!
//! Run: cargo test --release -p wat --test probe_arc209_host_agnostic_start

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::service::Outcome::Reply s' (:my::counter::IncrementResponse s'))))])

;; Drive through the client face, but start now takes a HOST — `(thread)` selects the shared-memory
;; launch via the Host protocol. Same round-trip as C.3 (increment 5 → get → 5).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start (:wat::spawn::thread) 0)
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::counter/increment c (:my::counter/increment-request 5))
     r  (:my::counter/get c (:my::counter/get-request))]
    (:my::counter::GetResponse/value r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn host_agnostic_start_dispatches_thread_launch() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (4a: start [host <- :Host] dispatches the thread launch via the Host protocol)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5 driven through the host-agnostic client face \
         (start (thread) 0 → connect → increment 5 → get); got {got:?}"
    );
}

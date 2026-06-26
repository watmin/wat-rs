//! Arc 209 locus-parity leg, stone 4a — the locus-agnostic `start [locus <- :Locus]`.
//!
//! C.3 shipped a THREAD-HARDCODED `start [state0]` (it bakes `(:wat::spawn::thread)` into the
//! `listener'` + `spawn-program'` calls, `wat/service.wat` start-body). Stone 4a makes `start` take
//! the locus as its first arg and route through the `:wat::spawn::Locus` protocol (arc 232) — so one
//! `start` serves any transport, dispatching the per-tier launch to the `extend-type :Locus` impl.
//! Thread is `extend-type :wat::spawn::ThreadOpts :Locus` (the shared-memory/capture strategy — the
//! `deftest'` model); process/remote join later as not-shared `extend-type`s, zero edit to `start`.
//!
//! THE GATE (thread-proven): the SAME counter as C.3, driven through the client face, but
//! `(:my::counter/start (:wat::spawn::thread) 0)` — start now takes a locus. The round-trip
//! (increment 5 → get → 5) proves the locus-agnostic start dispatches the thread launch correctly.
//!
//! RED at HEAD: C.3's `start` is arity-1 `[state0]` — `(counter/start (thread) 0)` is a 2-arg call
//! → arity mismatch; and `:wat::kernel::Locus` / the ThreadOpts `extend-type` don't exist. GREEN once
//! 4a ships the Locus protocol + thread impl + the locus-agnostic start codegen.
//!
//! Run: cargo test --release -p wat --test probe_arc209_locus_agnostic_start

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; start takes ::Record.
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State/new (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

;; Drive through the client face, but start now takes a LOCUS — `(thread)` selects the shared-memory
;; launch via the Locus protocol. Same round-trip as C.3 (increment 5 → get → 5).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h  (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record 0))
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::counter/increment c (:my::counter/increment-request 5))
     r  (:my::counter/get c (:my::counter/get-request))]
    (:my::counter::GetResponse/value r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn locus_agnostic_start_dispatches_thread_launch() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (4a: start [locus <- :Locus] dispatches the thread launch via the Locus protocol)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5 driven through the locus-agnostic client face \
         (start (thread) 0 → connect → increment 5 → get); got {got:?}"
    );
}

//! Arc 272 record-state rs-2 — a service's return value IS its final state: `(<svc>/stop c)` terminates
//! the service and yields its last state. THREAD tier here; the SAME mechanism serves process/remote.
//!
//! The lifecycle counterpart to `start`: `(<svc>/start locus state0) -> Handle` launches with an initial
//! state; `(<svc>/stop c) -> St` stops it and returns the final state (gen_server `terminate`-with-state).
//! Resumability falls out: this `final` is a valid `state0` for the next `start`.
//!
//! Mechanism (the build, not asserted here): the `:Stop` terminal op (gen_server `{stop, State}`).
//! defservice auto-generates a `stop` op + serve's `Outcome::Stop` arm (reply the final state to the
//! client, then EXIT the loop instead of recurring). `(<svc>/stop c)` sends the stop request over the
//! CLIENT connection and `recv'`s the final state AS THE REPLY — CONSTANT SHAPE across thread/process/
//! remote (it rides connect'/send'/recv', identical for every locus). No new substrate, no lineage
//! reshape. A crashed service makes the call RAISE (the existing recv' crash-surfacing) — sibling probe.
//!
//! RED at HEAD: defservice generates no `<fqdn>/stop` op (UnresolvedReference). GREEN once rs-2 ships the
//! `Outcome::Stop` variant + serve's stop arm + the generated stop op/method. `#[ignore]` until then.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_thread_stop_returns_final_state -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as a thread service; after incrementing, `stop` must return the accumulated final state.
// (state is i64 here — the strict state-must-be-a-record rule is arc-272 rs-1, deferred onto arc 273.)
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

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start (:wat::spawn::thread) 0)
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     final (:my::counter/stop c)]
    final))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn thread_stop_returns_the_services_final_state() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-2: thread stop returns final state)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: (:my::counter/stop h) stopped the thread service and returned its final state \
         (increment 5 set state 0→5); got {got:?}"
    );
}

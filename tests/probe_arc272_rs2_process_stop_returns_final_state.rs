//! Arc 272 record-state rs-2 — a service's return value IS its final state: `(<svc>/stop c)` terminates
//! the service and yields its last state. PROCESS tier here; identical mechanism to the thread tier.
//!
//! Proves `stop` is locus-agnostic: the reply rides `connect'`/`send'`/`recv'` unchanged across
//! thread and process. The final state crosses the socket (process tier) just as it crosses the
//! channel (thread tier) — constant client shape, no lineage reshape.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_process_stop_returns_final_state -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as a process service; after incrementing, `stop` must return the accumulated final state.
// (state is i64 here — the strict state-must-be-a-record rule is arc-272 rs-1, deferred onto arc 273.)
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state [count <- :wat::core::i64]
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::State/count s))))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ (:my::counter::State/count s) n)]
       (:wat::service::Outcome::Reply (:my::counter::State s') (:my::counter::IncrementResponse s'))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start (:wat::spawn::process) (:my::counter::State 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     final (:my::counter/stop c)]
    (:my::counter::State/count final)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_stop_returns_the_services_final_state() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rs-2: process stop returns final state)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: (:my::counter/stop h) stopped the process service and returned its final state \
         (increment 5 set state 0→5); got {got:?}"
    );
}

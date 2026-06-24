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
// arc 291 4b-ii: State is now a defstruct; :durable mints ::Record (the soul); stop returns ::Record.
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

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start (:wat::spawn::process) (:my::counter::Record 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     ;; arc 291 3a-ii-β: stop is now OWNER-ONLY — takes the Handle (h), not the client peer (c).
     ;; The final state rides UP the lineage channel (Status::Stopped), not the client reply.
     ;; arc 291 4b-ii: stop returns ::Record (the durable soul), read via Record/count.
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))

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

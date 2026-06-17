//! Arc 272 6b-ii-β — THE HEADLINE GATE: a `defservice` runs on a forked `(process)` through the
//! EXACT SAME client face it uses on a `(thread)`. This IS argspec parity made a test: the only
//! delta from `probe_arc209_c3_defservice_client_face` (thread, GREEN) is `(:wat::spawn::process)`
//! in place of `(:wat::spawn::thread)` — everything else (start, connect', the generated
//! increment/get methods, the request constructors, the Handle) is byte-identical.
//!
//! What 6b-ii-β must build for this to pass:
//!   - `Launched<S,R>{handle,address}` (spawn.wat) — what `Host/launch` returns.
//!   - `launch<S,R,St>` reshaped to mint the listener internally (via the now-working
//!     `(listener' self :S :R)`, the arc-232 dep) and return `Launched`; `start` unwraps it.
//!   - the ProcessOpts `launch` arm: spawn `<fqdn>::child-forms` → recv' the child-minted addr →
//!     send' state0 over the lineage (6b-ii-α) → return Launched.
//!   - defservice emits `<fqdn>::child-forms` (the Op/Reply enums + Request/Response records + serve
//!     + a `:user::main` driver), so the forked child's universe (stdlib + these forms) has the
//!     service code (the child runs a FRESH startup — it does NOT inherit the parent's defservice).
//!
//! The client face crosses Op/Reply RECORDS over the socket connection — proven possible by
//! 6b-ii-α (socket-tier recv' decodes records). state0 (0) crosses parent→child over the lineage.
//!
//! RED at HEAD: no ProcessOpts `launch` arm + no `child-forms` → starting on (process) fails.
//! GREEN when 6b-ii-β ships. `#[ignore]` until then.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6b_defservice_on_process -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// IDENTICAL to the C.3 thread probe except the host is (process). Parity = same client face.
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
    [h  (:my::counter/start (:wat::spawn::process) 0)
     c  (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _  (:my::counter/increment c (:my::counter/increment-request 5))
     r  (:my::counter/get c (:my::counter/get-request))]
    (:my::counter::GetResponse/value r)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_runs_on_a_forked_process_through_the_same_client_face() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (6b-ii-β: defservice on a process)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected GetResponse.value == 5 driven through the generated client face on a FORKED PROCESS \
         (start (process) 0 → connect → increment 5 → get); same face as the thread tier; got {got:?}"
    );
}

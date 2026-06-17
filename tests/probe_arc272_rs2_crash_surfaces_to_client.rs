//! Arc 272 record-state rs-2 (crash half) — a far-side crash SURFACES to the client as a raise, not a
//! hang or a fake value. The graceful/crash duality of the service contract: `(<svc>/stop c)` returns the
//! final state on a clean terminate; a crashed handler makes the client's call RAISE the crash reason.
//!
//! This is the EXISTING substrate crash-surfacing (peer.rs:110-123 thread crash channel; runtime.rs:23771
//! process Err channel → `PeerRecvError::Crashed` → `#wat.kernel/ProcessPanics`), exercised THROUGH the
//! generated client face: an op handler that crashes → the service dies → the client's `recv'` of the
//! reply raises the reason (deadlock-free — recv' checks the crash channel on output-EOF, never hangs).
//! gen_server-faithful: calling a dead server raises.
//!
//! Likely GREEN at HEAD already (the client face + crash-surfacing both exist) — this LOCKS that contract
//! as a tested invariant for the final-state feature; it must stay green after the `:Stop` work lands.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs2_crash_surfaces_to_client

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

// A service with one op whose handler CRASHES (assertion-failed! raises inside the serve loop).
const PROGRAM: &str = r#"
(:wat::service::defservice :my::svc
  :state [count <- :wat::core::i64]
  :ops
  [(:Boom [s <- :State]
          -> [ok <- :wat::core::bool]
     (:wat::kernel::assertion-failed!
       "boom — the handler crashed on purpose"
       (:wat::core::Some "boom")
       (:wat::core::Some "ok")))])

(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [h (:my::svc/start (:wat::spawn::thread) (:my::svc::State 0))
     c (:wat::kernel::connect' (:my::svc::Handle/addr h))
     _ (:my::svc/boom c (:my::svc/boom-request))]
    true))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn far_side_crash_raises_to_the_client_not_hang_or_fake() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (crash-surfacing probe)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    // The crashing handler must make the client's call RAISE (Err) — not return true, not hang.
    assert!(
        result.is_err(),
        "expected the far-side crash to RAISE to the client (recv' surfaces the crash reason); \
         instead the call returned: {result:?}"
    );
}

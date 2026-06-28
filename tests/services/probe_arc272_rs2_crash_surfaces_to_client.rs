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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn far_side_crash_raises_to_the_client_not_hang_or_fake() {
    // arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; start takes ::Record.
    // Wat source lives in the co-located fixture: probe_arc272_rs2_crash_surfaces_to_client.wat
    let world = startup_beside(file!())
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

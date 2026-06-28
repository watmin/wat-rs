//! Arc 209 C0b.3a-ii — the socket `poll'` service multiplexer (the process-tier service loop).
//!
//! C0b.1b built `poll'` (3-arg service multiplexer → `ServiceEvent`) THREAD-tier only.
//! C0b.3a-i shipped the process `Select` listener-arm + poll-driven non-blocking accept.
//! C0b.2e-ii made `Listener` a proper transport-blind entity. This stone adds the PROCESS
//! branch to `poll'`: a spawned `(process)` service multiplexes its self-peer + a socket
//! listener + N socket client peers over ONE `process::Select` ring → `ServiceEvent`.
//!
//! THE GATE (this probe IS the process service proof — and the DEADLOCK gate): a spawned
//! `(process)` service autobinds a listener (no name — unguessable capability), sends its
//! minted `Address'` to its owner over the self-peer (arc 272 capability handoff — race-free,
//! no sleep, no fixed name), then `poll'`-loops — `:Connection`→grow, `:Message`→
//! echo n+100 + reply, `:Closed`→shrink, `:Shutdown`→exit. The PARENT waits for the
//! capability, `connect'`s to the minted address, round-trips 5→105, then simply DROPS the
//! service handle at scope-exit. The deadlock-free termination: dropping the handle → the
//! child's input pipe EOFs → the self-peer's `Recv{0}` fires → `poll'` returns `:Shutdown` →
//! the loop exits → the child ends → the owner's join completes. **No cooperative Stop —
//! dropping the handle IS the shutdown.** If this hangs, `poll'` isn't watching the self-peer
//! over the socket tier — the exact deadlock this stone must annihilate.
//!
//! RED at HEAD (pre-272): the service bound by NAME (socket-address') — the guessable name
//! is eliminated; now autobind+capability-handoff (arc 272 step 5). The proof is preserved:
//! service loop runs, round-trips, terminates on owner-drop.
//!
//! GREEN proves BOTH serve AND termination: `compute` returns 105 only after `svc` drops at
//! scope-exit, and the process handle's Drop joins the child — so if the loop did NOT
//! terminate on owner-drop (the deadlock), that join would hang and 105 would never return.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn process_service_loop_polls_serves_and_terminates_on_owner_drop() {
    // Wat source lives in the co-located fixture: probe_arc209_c0b3aii_process_service_loop.wat
    let world = startup_beside(file!())
        .expect("startup should succeed (C0b.3a-ii: socket poll' service multiplexer)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 round-tripped through the spawned process service's poll'-loop \
         (client sends 5 → service replies n+100), and the service terminated cleanly when \
         the owner dropped the handle (no hang); got {got:?}"
    );
}

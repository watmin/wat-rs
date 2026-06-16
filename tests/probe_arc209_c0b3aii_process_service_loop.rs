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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             ;; ── the service loop (named recursion) — poll' multiplexes the self-peer
             ;; (owner link → :Shutdown), the socket listener (new connections), and the
             ;; connected socket client peers (requests) over ONE process::Select ring. ──
             (:wat::core::defn :user::serve
               [self    <- :wat::kernel::Peer'<wat::kernel::Address'<wat::core::i64,wat::core::i64>,wat::core::i64>
                l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                 ;; SHUTDOWN — owner dropped the handle; RAII drain EOF'd the self-peer.
                 ;; Return nil → the loop exits, clients drop, the child ends, join completes.
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ;; GROW — poll' accepted the dialing client; conj the new peer.
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer)))
                 ;; SERVE — an i64 arrived from clients[idx]; reply n+100.
                 ((:wat::spawn::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                          (:wat::core::+ n 100))]
                     (:user::serve self l clients)))
                 ;; SHRINK — clients[idx] left gracefully.
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))
                 ;; SHRINK — clients[idx]'s transport broke (remote tier; cause is a Failure).
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))))
             ;; the child entry: autobind (no name — unguessable capability), hand the minted
             ;; address to the parent over the self-peer (arc 272 capability handoff), then serve.
             ;; The self-peer carries Address'<i64,i64> up to the parent (S), i64 down from parent (R).
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :wat::core::i64)
                  _    (:wat::kernel::send' self (:wat::spawn::Bound/address b))]
                 (:user::serve self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::kernel::recv' svc)
     ;; dial the capability — the child is guaranteed listening (it sent AFTER listen()).
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::kernel::recv' c)]
    ;; No Stop op. Scope-exit drops `svc` → the child's input pipe EOFs → the self-peer's
    ;; Recv{0} fires → serve's poll' returns :Shutdown → the child exits → join completes.
    ;; (If this hangs, poll' isn't watching the self-peer over the socket tier — STOP-1.)
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_service_loop_polls_serves_and_terminates_on_owner_drop() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
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

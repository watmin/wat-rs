//! Arc 272 step 6b-i — DISCONFIRMING PROBE: initial state crosses the fork PARENT→CHILD over the
//! lineage channel, and the process serve loop threads it.
//!
//! The grounded crux (the only genuinely-unproven bit 6b needs): a process service must receive its
//! initial `state0` from its owner. Already proven on disk, NOT re-probed here:
//!   - the process `poll'` serve loop + owner-drop termination — `probe_arc209_c0b3aii_process_service_loop`.
//!   - a record crossing the fork CHILD→PARENT over the lineage channel — `probe_arc272_6c2_record_ipc_derisk`.
//! Every existing test sends child→parent over the lineage (the minted Address') and parent→child only
//! over a SEPARATE socket. This probe isolates the missing direction: **parent→child over the lineage
//! channel** — `(send' svc state0)` reaching the child's `(recv' self)`.
//!
//! The service is a counter whose reply DERIVES from the crossed state: child replies `base + n`, where
//! `base` lives only in the `state0` record the parent sends. So `5 -> 1005` is only possible if the
//! `Counter{base 1000}` actually crossed parent→child over the lineage — a wrong/missing state could
//! not produce 1005. Design B3, DESIGN-STONE-6b-process-launch.md.
//!
//! Was RED before 6b-ii-α; now GREEN (the regression test). The real failure was decode-side: the
//! socket-tier `recv' self` arm decoded via `peer.recv()` with NO type registry, so the child's
//! `(recv' self)` raised `NoTypeRegistry` on the `#user/Counter` tag and exited (the parent's send
//! had already landed; `connect'` then found the listener gone — "Connection refused"). 6b-ii-α routes
//! socket-tier `recv'` through `recv_wire()` + `decode_trusted_wire(sym.types())`, mirroring the
//! PROCESS arm and symmetric with the encode-in-eval send side (258.5b-ii).
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6b_state_over_lineage

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::Record::def :user::Counter [base <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             ;; the child runs a FRESH startup (stdlib + these forms only) — the record must be
             ;; defined here too so the crossed Counter reconstructs in the child universe.
             (:wat::Record::def :user::Counter [base <- :wat::core::i64])
             ;; the serve loop, now threading `state` (the Counter): reply derives base + n.
             (:wat::core::defn :user::serve
               [self    <- :wat::kernel::Peer'<wat::kernel::Address'<wat::core::i64,wat::core::i64>,user::Counter>
                l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>
                state   <- :user::Counter]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer) state))
                 ;; SERVE — reply base + n, where base lives only in the crossed state0.
                 ((:wat::spawn::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                          (:wat::core::+ (:user::Counter/base state) n))]
                     (:user::serve self l clients state)))
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::std::list::remove-at clients idx) state))
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::std::list::remove-at clients idx) state))))
             ;; the child entry: autobind (no name), hand the capability up (child→parent, proven),
             ;; then RECEIVE state0 down from the parent over the lineage (parent→child — the gap),
             ;; then serve with it. self-peer: S = Address' (up), R = Counter (down).
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                          :wat::kernel::Address'<wat::core::i64,wat::core::i64> :user::Counter)
                  _    (:wat::kernel::send' self (:wat::spawn::Bound/address b))
                  st   (:wat::kernel::recv' self)]
                 (:user::serve self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>) st)))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::kernel::recv' svc)
     ;; hand the child its initial state over the lineage channel (parent→child — the NEW direction).
     _    (:wat::kernel::send' svc (:user::Counter 1000))
     ;; dial the capability; round-trip 5 -> base + 5 == 1005 (only if state0 crossed).
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::kernel::recv' c)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn initial_state_crosses_parent_to_child_over_lineage_and_serve_threads_it() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (6b-i: state0 over the lineage channel)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(1005)),
        "expected 1005: the parent sent Counter{{base 1000}} to the child over the LINEAGE channel \
         (parent→child), the child recv'd it and threaded it through serve, replying base + 5 = 1005; \
         got {got:?}"
    );
}

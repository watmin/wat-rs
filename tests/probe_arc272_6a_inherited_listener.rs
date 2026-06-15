//! Arc 272 step 6a — the child accepts on a PARENT-MINTED listener it INHERITED (no name).
//!
//! This is the c0b3aii process-service-loop probe with the rendezvous NAME annihilated. In
//! c0b3aii the child binds its own listener by a fixed abstract name
//! (`socket-address' "wat.arc209.c0b3aii.svc"`) and the parent dials that same name — the
//! collidable + forgeable global string arc 272 exists to delete. Here:
//!   - the PARENT autobinds `(listener' (process) :i64 :i64)` → `Bound` (step 2b) — a kernel-minted,
//!     unguessable address, NO chosen name;
//!   - the parent HANDS the listener to the child via `(spawn-program' (process) l <forms>)`;
//!   - the child reads its inherited listener via `(:wat::program::listener :i64 :i64)` — the exact
//!     mirror of `(:wat::program::self-peer …)` — never a name;
//!   - the parent dials the minted CAPABILITY `(connect' addr)`, round-trips 5→105, and drops the
//!     handle to terminate (RAII shutdown, same as c0b3aii).
//!
//! RED at HEAD: the listener-inheritance surface does not exist yet — the process clause of
//! `spawn-program'` does not take a listener arg, and `:wat::program::listener` is unknown. The
//! program fails to type-check / startup (the first unbuilt rung), so `compute` never returns 105.
//! GREEN once 6a wires `install_listener` + the `:wat::program::listener` accessor + the
//! `spawn-program'(process)` listener arg (dup2→fd3 + `extra_preserved`).
//!
//! GREEN proves BOTH serve AND clean owner-drop termination: 105 returns only after `svc` drops at
//! scope-exit (the handle's Drop joins the child); if the loop did not terminate on owner-drop, the
//! join would hang and 105 would never return.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6a_inherited_listener

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    ;; ── PARENT mints the rendezvous: autobind, no name. The listener is the capability. ──
    [b    (:wat::kernel::listener' (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener b)
     addr (:wat::spawn::Bound/address b)
     ;; ── the child INHERITS the listener: handed in as an explicit arg to spawn-program'(process). ──
     svc  (:wat::kernel::spawn-program' (:wat::spawn::process) l
            (:wat::core::forms
              ;; the service loop — identical to c0b3aii (poll' multiplexes self-peer + listener + clients).
              (:wat::core::defn :user::serve
                [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
                 l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                 clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
                -> :wat::core::nil
                (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                  (:wat::spawn::ServiceEvent::Shutdown nil)
                  ((:wat::spawn::ServiceEvent::Connection peer)
                    (:user::serve self l (:wat::core::conj clients peer)))
                  ((:wat::spawn::ServiceEvent::Message idx n)
                    (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                           (:wat::core::+ n 100))]
                      (:user::serve self l clients)))
                  ((:wat::spawn::ServiceEvent::Closed idx)
                    (:user::serve self l (:wat::std::list::remove-at clients idx)))
                  ((:wat::spawn::ServiceEvent::Lost idx _cause)
                    (:user::serve self l (:wat::std::list::remove-at clients idx)))))
              ;; the child entry: get the INHERITED listener (no socket-address'/name), signal READY, serve.
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  [l    (:wat::program::listener :wat::core::i64 :wat::core::i64)
                   self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                   _    (:wat::kernel::send' self 1)]
                  (:user::serve self l
                    (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))))
     _    (:wat::kernel::recv' svc)
     ;; dial the minted CAPABILITY, not a name.
     c    (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c 5)
     got  (:wat::kernel::recv' c)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn child_accepts_on_inherited_parent_minted_listener() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (6a: inherited-listener surface)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 round-tripped through a process service that accepted on a PARENT-MINTED \
         listener it INHERITED (no name): parent autobinds → hands l to spawn-program'(process) → \
         child reads (:wat::program::listener) → poll'-serves → parent dials (connect' addr) → \
         5 replies 105; and the service terminated cleanly on owner-drop; got {got:?}"
    );
}

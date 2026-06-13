//! Arc 209 C0b.1b — `select'` is the service multiplexer + the `SelectEvent<I,O>` sum.
//!
//! THE GATE (this probe IS the hand-rolled thread service proof): a service `select'`s over
//! THREE inputs — the **self-peer** (owner link), the **listener**, the **clients** — and
//! `match`es the returned `SelectEvent`:
//!   - `:Shutdown`          → owner dropped the handle (RAII drain disconnected the self-peer);
//!                            exit the loop (DEADLOCK-FREE TERMINATION — structural, no Stop op)
//!   - `:Connection [peer]` → `select'` accepted the dialing client; conj it (GROW)
//!   - `:Message [idx msg]` → handle the op on `clients[idx]`, reply, recur (SERVE)
//!   - `:Closed [idx]`      → `remove-at` (graceful SHRINK)
//!   - `:Lost [idx cause]`  → `remove-at` (abnormal SHRINK; remote tier; `cause` is a Failure)
//! Two clients dynamically connect, each round-trips a protected scalar (n*2); then the owner
//! simply DROPS the service handle at scope-exit → `:Shutdown` → the service terminates and the
//! join completes. No cooperative stop — dropping the handle IS the shutdown. (If this hangs,
//! `select'` isn't watching the self-peer — the deadlock this stone annihilates.)
//!
//! RED at HEAD: the 3-arg `select'` form and `:wat::kernel::SelectEvent` do not exist — the
//! program fails to type-check on exactly that gap. GREEN once C0b.1b ships the 3-arg `select'`
//! returning `SelectEvent<I,O>`.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; The client op protocol: compute-and-reply. No Stop op — the service is
;; terminated STRUCTURALLY by the owner dropping its handle (→ :Shutdown), never
;; by a cooperative message.
(:wat::core::defenum :user::Op
  :Compute [n <- :wat::core::i64])

;; The service loop — named recursion. select' watches THREE inputs: the self-peer
;; (owner/supervisor link → :Shutdown), the listener (new connections), and the
;; connected client server-ends (requests). One blocking call multiplexes all three.
(:wat::core::defn :user::serve
  [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
   l       <- :wat::kernel::Listener'<user::Op,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,user::Op>>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::select' self l clients) -> :wat::core::nil
    ;; SHUTDOWN — the owner dropped the service handle; RAII drain disconnected the
    ;; self-peer → select' fired :Shutdown. Return nil → the loop exits, clients drop,
    ;; the thread ends, the owner's join completes. The deadlock-free guarantee:
    ;; dropping the handle ALWAYS terminates the loop, structurally, no cooperation.
    (:wat::kernel::SelectEvent::Shutdown nil)
    ;; GROW — select' accepted the dialing client and hands the new peer back; add it.
    ((:wat::kernel::SelectEvent::Connection peer)
      (:user::serve self l (:wat::core::conj clients peer)))
    ;; SERVE — an op arrived from clients[idx].
    ((:wat::kernel::SelectEvent::Message idx msg)
      (:wat::core::match msg -> :wat::core::nil
        ((:user::Op::Compute n)
          (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                 (:wat::core::* n 2))]
            (:user::serve self l clients)))))
    ;; SHRINK — clients[idx] left gracefully (clean disconnect, no diagnostic).
    ((:wat::kernel::SelectEvent::Closed idx)
      (:user::serve self l (:wat::std::list::remove-at clients idx)))
    ;; SHRINK — clients[idx]'s transport broke abnormally; `cause` is the first-class
    ;; diagnostic (a Failure). Emitted by the remote tier; the thread tier never
    ;; raises this, but the arm is built for the union.
    ((:wat::kernel::SelectEvent::Lost idx _cause)
      (:user::serve self l (:wat::std::list::remove-at clients idx)))))

;; Spawn the service, connect two clients dynamically, round-trip a scalar through each,
;; then Stop. Returns r1 + r2 (expect 10 + 14 = 24).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :user::Op :wat::core::i64)
     l    (:wat::core::first pair)
     addr (:wat::core::second pair)
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:user::serve self l (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,user::Op>))))
     c1   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c1 (:user::Op::Compute 5))
     r1   (:wat::kernel::recv' c1)
     c2   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c2 (:user::Op::Compute 7))
     r2   (:wat::kernel::recv' c2)]
    ;; No Stop op. Scope-exit drops `svc` → RAII drain disconnects the self-peer →
    ;; serve's select' fires :Shutdown → the service exits → the owner's join completes.
    ;; Dropping the handle IS the shutdown. (If this deadlocks, select' isn't watching
    ;; the self-peer — the exact bug this stone annihilates.)
    (:wat::core::+ r1 r2)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn select_grows_over_listener_serves_and_shrinks() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(24)),
        "expected r1+r2 = 10+14 = 24 (two dynamically-connected clients each round-tripped n*2); got {got:?}"
    );
}

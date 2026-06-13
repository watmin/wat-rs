//! Arc 209 C0b.1b — `select'` learns the `Listener'` + the `SelectEvent<O>` sum.
//!
//! THE GATE (this probe IS the hand-rolled thread service proof): a service `select'`s over
//! `(listener, clients)` and `match`es the returned `SelectEvent`:
//!   - `:Connection`        → `accept'` the dialing client → conj onto the clients (GROW)
//!   - `:Message [idx msg]` → handle the op on `clients[idx]`, reply, recur (SERVE)
//!   - `:Closed [idx]`      → `remove-at` (graceful SHRINK)
//!   - `:Crashed [idx r]`   → `remove-at` (SHRINK + diagnostics)
//! Two clients dynamically connect, each round-trips a protected scalar (n*2), then an explicit
//! `Stop` op exits the loop (Stone C/D owns owner-RAII shutdown; the probe uses Stop so the
//! spawned service thread terminates and the test does not hang).
//!
//! RED at HEAD: the 2-arg `select'` form and `:wat::kernel::SelectEvent` do not exist — the
//! program fails to type-check on exactly that gap. GREEN once C0b.1b ships the 2-arg `select'`
//! returning `SelectEvent<O>`.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; The client op protocol: compute-and-reply, or stop the service.
(:wat::core::defenum :user::Op
  :Compute [n <- :wat::core::i64]
  :Stop)

;; The service loop — named recursion over (listener, connected clients).
;; Watches the listener (new connections) + the client server-ends; ignores its
;; own self-peer (the service is stopped by the Stop op, not by owner-drop — C0b.1b).
(:wat::core::defn :user::serve
  [l       <- :wat::kernel::Listener'<user::Op,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,user::Op>>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::select' l clients) -> :wat::core::nil
    ;; GROW — a client is dialing; accept it onto the set.
    (:wat::kernel::SelectEvent::Connection
      (:user::serve l (:wat::core::conj clients (:wat::kernel::accept' l))))
    ;; SERVE — an op arrived from clients[idx].
    ((:wat::kernel::SelectEvent::Message idx msg)
      (:wat::core::match msg -> :wat::core::nil
        ((:user::Op::Compute n)
          (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                 (:wat::core::* n 2))]
            (:user::serve l clients)))
        (:user::Op::Stop nil)))
    ;; SHRINK — clients[idx] left gracefully.
    ((:wat::kernel::SelectEvent::Closed idx)
      (:user::serve l (:wat::std::list::remove-at clients idx)))
    ;; SHRINK — clients[idx] died; reason available (diagnostics).
    ((:wat::kernel::SelectEvent::Crashed idx _reason)
      (:user::serve l (:wat::std::list::remove-at clients idx)))))

;; Spawn the service, connect two clients dynamically, round-trip a scalar through each,
;; then Stop. Returns r1 + r2 (expect 10 + 14 = 24).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :user::Op :wat::core::i64)
     l    (:wat::core::first pair)
     addr (:wat::core::second pair)
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [_self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:user::serve l (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,user::Op>))))
     c1   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c1 (:user::Op::Compute 5))
     r1   (:wat::kernel::recv' c1)
     c2   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c2 (:user::Op::Compute 7))
     r2   (:wat::kernel::recv' c2)
     _    (:wat::kernel::send' c1 (:user::Op::Stop))]
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

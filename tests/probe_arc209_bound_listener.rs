//! Arc 209 (protocol tooling, sub-stone 1) — `listener'` (thread tier) returns a
//! `Bound<S,R>` struct instead of the bare `Tuple<Listener'<S,R>, Address'<S,R>>`.
//!
//! `Bound` is a parametric STRUCT (not a record) because its fields are non-EDN
//! RustOpaque kernel entities (`Listener'`/`Address'`):
//!   (:wat::core::defstruct :wat::kernel::Bound<S,R>
//!     [listener <- :wat::kernel::Listener'<S,R>
//!      address  <- :wat::kernel::Address'<S,R>])
//! The thread tier of `listener'` builds it; the accessors `Bound/listener` and
//! `Bound/address` replace the positional `first`/`second` on the old tuple.
//!
//! This probe is `probe_arc209_c0b1b_select_listener` reduced to a single client,
//! with EXACTLY two lines changed: `(first pair)` → `(:wat::kernel::Bound/listener b)`
//! and `(second pair)` → `(:wat::kernel::Bound/address b)`. So a failure isolates
//! precisely to `Bound` — everything around it is the proven c0b1b round-trip.
//!
//! RED at HEAD: `:wat::kernel::Bound` is unregistered (no `defstruct`) AND `listener'`
//! returns a `Tuple` — so the `Bound/listener` / `Bound/address` accessors do not
//! resolve and the program fails to check on exactly that gap. GREEN once the
//! `defstruct` ships in `wat/spawn.wat` and `eval_listener_prime`'s thread tier
//! returns `Value::Struct{ ":wat::kernel::Bound", [listener, address] }`.
//!
//! Run SERIALLY (spawns a thread):
//!   cargo test --release -p wat --test probe_arc209_bound_listener -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; The client op protocol: compute-and-reply. No Stop op — the owner dropping its
;; handle (→ :Shutdown) terminates the service structurally (the c0b1b guarantee).
(:wat::core::defenum :user::Op
  :Compute [n <- :wat::core::i64])

;; The service loop — poll' multiplexes the self-peer, the listener, the clients.
(:wat::core::defn :user::serve
  [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
   l       <- :wat::kernel::Listener'<user::Op,wat::core::i64>
   clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,user::Op>>]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
    (:wat::kernel::ServiceEvent::Shutdown nil)
    ((:wat::kernel::ServiceEvent::Connection peer)
      (:user::serve self l (:wat::core::conj clients peer)))
    ((:wat::kernel::ServiceEvent::Message idx msg)
      (:wat::core::match msg -> :wat::core::nil
        ((:user::Op::Compute n)
          (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                 (:wat::core::* n 2))]
            (:user::serve self l clients)))))
    ((:wat::kernel::ServiceEvent::Closed idx)
      (:user::serve self l (:wat::std::list::remove-at clients idx)))
    ((:wat::kernel::ServiceEvent::Lost idx _cause)
      (:user::serve self l (:wat::std::list::remove-at clients idx)))))

;; Spawn the service, connect one client, round-trip a scalar (5*2 = 10), then
;; scope-exit drops `svc` → :Shutdown → the service terminates and the join completes.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [b    (:wat::kernel::listener' (:wat::spawn::thread) :user::Op :wat::core::i64)
     l    (:wat::kernel::Bound/listener b)
     addr (:wat::kernel::Bound/address b)
     svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:user::serve self l (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,user::Op>))))
     c1   (:wat::kernel::connect' addr)
     _    (:wat::kernel::send' c1 (:user::Op::Compute 5))
     r1   (:wat::kernel::recv' c1)]
    r1))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn listener_thread_tier_returns_bound_struct() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Bound defstruct + listener' thread tier returns Bound)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(10)),
        "expected 10 = 5*2: Bound/listener fed serve's poll', Bound/address dialed the client, \
         the round-trip succeeded; got {got:?}"
    );
}

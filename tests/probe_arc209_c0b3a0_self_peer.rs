//! Arc 209 C0b.3a-0 — the process self-peer verb `(:wat::program::self-peer :S :R)`.
//!
//! A process SERVICE must hold its owner-link as a `Peer'` to pass to `select'` (arg0) and watch
//! for `:Shutdown`. The thread tier hands the self-peer as a fn arg; the process tier CANNOT
//! (separate memory carries serialized forms, not live handles — arc 213). So the process child
//! obtains its own self-peer from its inherited fds via a verb:
//!   `(:wat::program::self-peer :S :R) -> SocketPeer'<S,R>` — rx over fd 0 (owner→child; EOF =
//!   `:Shutdown`), tx over fd 1 (child→owner). Installed only at the spawned-child seam
//!   (`run_forms_as_server_child`); in root it is a clean error (no owner-link).
//!
//! THE GATE — an echo service via the self-peer, proving BOTH directions of the owner-link:
//! the parent `send'`s 5 to the spawned `Process'` handle → the child `recv'`s it on its self-peer
//! (rx = fd 0) → adds 100 → `send'`s 105 back on its self-peer (tx = fd 1) → the parent `recv'`s 105
//! from the handle. Proves: the verb returns a working self-peer wrapping the child's fd0/fd1.
//!
//! RED at HEAD: `(:wat::program::self-peer …)` does not exist → the program fails to type-check on
//! exactly that gap. GREEN once C0b.3a-0 ships the verb + the child-seam install.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary (auto-registered).
//! Run: cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; Parent spawns a process echo service; the child gets its self-peer and echoes owner→child + 100.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                  x    (:wat::kernel::recv' self)
                  _    (:wat::kernel::send' self (:wat::core::+ x 100))]
                 nil))))
     _   (:wat::kernel::send' svc 5)
     got (:wat::kernel::recv' svc)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_self_peer_echoes_over_the_owner_link() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3a-0: process self-peer verb)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 echoed over the process self-peer (parent send' 5 → child recv' self → \
         send' self 105 → parent recv'); got {got:?}"
    );
}

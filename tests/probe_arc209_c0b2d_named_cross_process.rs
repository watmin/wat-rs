//! Arc 209 C0b.2d — connect-by-name: the first TRUE cross-process connection.
//!
//! C0b.2c's `listener'` (process) MINTS a unique name and returns a `SocketAddress'` OPAQUE — which
//! can't cross a process boundary (opaques aren't EDN). So a separately-spawned client can't learn
//! where to `connect'`. C0b.2d fixes it: both parties rendezvous by a shared NAME (a String literal),
//! constructed into a typed address via `(:wat::kernel::socket-address' name :S :R)`. The service
//! BINDS it (`(listener' (process) addr)`); the client DIALS it (`(connect' addr)` — unchanged).
//!
//! THE GATE: a spawned process service binds the known name, signals READY to its owner over the
//! self-peer (C0b.3a-0 — race-free, no sleep), accepts the parent's connection, and echoes +100.
//! The PARENT (a separate process) waits for READY, then `connect'`s by the SAME name and round-trips.
//! 5 → 105 across the process boundary, rendezvoused by name. (No `select'` — that's C0b.3a-ii.)
//!
//! RED at HEAD: `socket-address'` doesn't exist + `listener' (process)` still mints (the 2-arg
//! bind-addr form is unknown) → the child fails startup type-check → `recv'` raises. GREEN once
//! C0b.2d ships `socket-address'` + the `listener' (process) addr` bind form.
//!
//! This test FORKS (spawn-program' (process)) → its own top-level [[test]] binary (auto-registered).
//! Run: cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [l    (:wat::kernel::listener' (:wat::spawn::process)
                         (:wat::kernel::socket-address' "wat.arc209.c0b2d.svc" :wat::core::i64 :wat::core::i64))
                  _    (:wat::kernel::send' (:wat::program::self-peer :wat::core::i64 :wat::core::i64) 1)
                  cli  (:wat::kernel::accept' l)
                  x    (:wat::kernel::recv' cli)
                  _    (:wat::kernel::send' cli (:wat::core::+ x 100))]
                 nil))))
     _   (:wat::kernel::recv' svc)
     c   (:wat::kernel::connect'
           (:wat::kernel::socket-address' "wat.arc209.c0b2d.svc" :wat::core::i64 :wat::core::i64))
     _   (:wat::kernel::send' c 5)
     got (:wat::kernel::recv' c)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn named_address_round_trips_across_a_process_boundary() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.2d: connect-by-name)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 round-tripped across the process boundary, rendezvoused by the shared name \
         (parent connect' by name → service accept' → echo +100); got {got:?}"
    );
}

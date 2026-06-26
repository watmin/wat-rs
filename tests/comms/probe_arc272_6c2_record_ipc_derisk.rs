//! Arc 272 6c.2 DE-RISK — does a plain user `Record::def` round-trip over the process-fork IPC wire?
//!
//! This isolates the exact seam the 6c.2 (D1) crashed/orphaned strike tripped on: a RECORD carried
//! across `send'`/`recv'` after a `spawn-program' (process)` fork. D1 makes the address capability a
//! `SocketAddressWire` record; its whole foundation is "a user record survives the process IPC wire."
//! 234.7a proved records round-trip via `value_to_edn`/`edn_to_value` IN-PROCESS; this proves it ACROSS
//! a fork (the new axis). GREEN ⇒ D1's foundation is sound, the orphan's failure was implementation.
//! RED ⇒ records do not cross the fork wire (a deeper plumbing gap), and D1 is blocked until that's fixed.
//!
//! Forks (`spawn-program' (process)`) → its own [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6c2_record_ipc_derisk

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
(:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
            (:wat::core::forms
              ;; The forked child runs a FRESH startup (stdlib prelude + these forms only) — it does
              ;; NOT inherit the parent's top-level defs. So the record must be defined HERE too (D1's
              ;; SocketAddressWire avoids this by living in spawn.wat/stdlib, loaded in every universe).
              (:wat::core::defrecord :user::Pt [x <- :wat::core::i64  y <- :wat::core::i64])
              (:wat::core::defn :user::main [] -> :wat::core::nil
                (:wat::core::let
                  ;; the child mints a plain base record and hands it to the parent over the self-peer.
                  [self (:wat::program::self-peer :user::Pt :wat::core::i64)
                   _    (:wat::kernel::send' self (:user::Pt 7 35))]
                  nil))))
     ;; the parent recv's the record off the lineage channel; reconstruct via the EDN wire.
     pt  (:wat::kernel::recv' svc)]
    (:wat::core::+ (:user::Pt/x pt) (:user::Pt/y pt))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn plain_record_round_trips_over_process_ipc() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (de-risk: record over process IPC)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: the child minted a :user::Pt{{7,35}}, sent it over the self-peer, the parent \
         recv'd it across the fork and read x+y. If this is RED, a plain record does NOT round-trip \
         over the process IPC wire — D1 is blocked on that, not on the capability codec. got {got:?}"
    );
}

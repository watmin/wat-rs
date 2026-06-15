//! Arc 209 (host-parity leg, stone 2) — the `:wat::kernel::Spawned` handle marker.
//!
//! The uniform bound the host-agnostic defservice `Handle` field needs: the spawn handles
//! `Thread'<I,O>`/`Process'<I,O>` (and future remote handles) all `derive` it, so `Handle.handle`
//! can hold any of them. It is a typesub/`isa?` marker (Clojure's `derive` axis) — NO methods, NOT a
//! protocol. The stone adds, in `wat/spawn.wat`:
//!   (:wat::core::derive :wat::kernel::Thread'  :wat::kernel::Spawned)
//!   (:wat::core::derive :wat::kernel::Process' :wat::kernel::Spawned)
//! and retypes defservice's `Handle.handle` from `Thread'<Op,Reply>` to `:wat::kernel::Spawned`.
//!
//! This probe: a real `Thread'` from `spawn-program'` flows to a `:wat::kernel::Spawned`-typed param.
//! It exercises the STDLIB marker (not an inline declaration) — the derive edges live in spawn.wat.
//! Builds on the shipped `derive` verb (arc 237 follow-on) + parametric protocol bounds (arc 267 —
//! `Thread'<I,O>` is parametric, so the `assignable` head-arm carries the satisfaction).
//!
//! RED at HEAD: `wat/spawn.wat` does not yet derive `Thread'`→`:Spawned`, so `Thread'<i64,i64>` is
//! not a subtype of `:wat::kernel::Spawned` → the bound rejects it. GREEN once the stone adds the
//! derive edges.
//!
//! Run SERIALLY (spawns a thread):
//!   cargo test --release -p wat --test probe_arc209_spawned_marker -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; A fn bound over the stdlib spawn-handle marker — accepts any handle that derives :Spawned.
(:wat::core::defn :user::take-spawned [h <- :wat::kernel::Spawned] -> :wat::core::i64 99)

;; Get a real Thread' from spawn-program' (thread tier) and pass it through the :Spawned bound.
(:wat::core::defn :user::go [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::thread)
           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
             nil))]
    (:user::take-spawned svc)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn thread_handle_derives_the_spawned_marker() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Thread' derives :wat::kernel::Spawned via spawn.wat)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(99)),
        "expected 99: a Thread' (which derives :wat::kernel::Spawned in spawn.wat) must be accepted \
         where the :Spawned marker bound is required; got {got:?}"
    );
}

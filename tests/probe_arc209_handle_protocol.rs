//! Arc 209 (host-parity leg, stone 1) — the spawn-handle protocol over the BUILT-IN OPAQUES.
//!
//! The host-agnostic `start` needs a uniform bound its spawn handle satisfies, so defservice's
//! `Handle.handle` field can hold either a `Thread'` or a `Process'` and a remote handle drops in
//! by `extend-type` (zero central edit). The mechanism is arc 232 defprotocol/extend-type — but
//! arc 232's probes only `extend-type`'d user `Record`s (Robot/Dog). THE NOVEL RISK this probe
//! isolates: does a protocol bound + method dispatch work when the extender is a **built-in opaque
//! parametric** type — `:wat::kernel::Thread'<I,O>` — produced by `spawn-program'`?
//!
//! Shape (232.3 applied to the spawn handle): a protocol `:wat::kernel::Spawned` with one method;
//! `extend-type` it onto `Thread'` (and `Process'`); a fn typed `[h <- :Spawned]` accepts a real
//! `Thread'` value from `spawn-program'` and dispatches the method on its concrete type.
//!
//! RED at HEAD: `:wat::kernel::Spawned` does not exist → `defprotocol`/`extend-type`/the bound all
//! fail; the program won't check. GREEN once the handle protocol ships and `extend-type` registers
//! the subtype edge for the opaque `Thread'` (register_subtype is name-based, so this should hold —
//! the probe proves the assignable + dispatch path handles the opaque head).
//!
//! Run SERIALLY (spawns a thread):
//!   cargo test --release -p wat --test probe_arc209_handle_protocol -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; The uniform spawn-handle bound. Thread'/Process'/future-remote extend-type it; a remote drops in
;; with one extend-type, zero central edit (the open-seam doctrine, via arc 232).
(:wat::core::defprotocol :wat::kernel::Spawned
  (spawned-tag [self <- :wat::kernel::Spawned] -> :wat::core::String))

;; extend-type onto the BUILT-IN OPAQUES (the novel part — 232 only did user Records).
(:wat::core::extend-type :wat::kernel::Thread'  :wat::kernel::Spawned (spawned-tag [self] "thread"))
(:wat::core::extend-type :wat::kernel::Process' :wat::kernel::Spawned (spawned-tag [self] "process"))

;; A fn typed over the handle bound — accepts any extender, dispatches on the concrete handle type.
(:wat::core::defn :user::tag-of [h <- :wat::kernel::Spawned] -> :wat::core::String
  (:wat::kernel::Spawned/spawned-tag h))

;; Get a REAL Thread' from spawn-program' (thread tier) and pass it through the :Spawned bound.
;; The self-peer prog is trivial; the handle drops via RAII at scope exit.
(:wat::core::defn :user::go [] -> :wat::core::String
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::thread)
           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
             nil))]
    (:user::tag-of svc)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn handle_protocol_binds_and_dispatches_over_builtin_opaque() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (handle protocol :Spawned extend-typed onto Thread'/Process')");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(&got, Value::String(s) if s.as_str() == "thread"),
        "expected \"thread\": a Thread' (built-in opaque) passed through a :Spawned-typed param must \
         dispatch spawned-tag to the Thread' impl; got {got:?}"
    );
}

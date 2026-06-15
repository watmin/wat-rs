//! Arc 209 (host-parity leg, stone 1) — the spawn-handle protocol over the BUILT-IN OPAQUES.
//!
//! The host-agnostic `start` needs a uniform bound its spawn handle satisfies, so defservice's
//! `Handle.handle` field can hold either a `Thread'` or a `Process'` and a remote handle drops in
//! by `extend-type` (zero central edit). The mechanism is arc 232 defprotocol/extend-type — but
//! arc 232's probes only `extend-type`'d user `Record`s (Robot/Dog). THE NOVEL RISK this probe
//! isolates: does a protocol bound + method dispatch work when the extender is a **built-in opaque
//! parametric** type — `:wat::kernel::Thread'<I,O>` — produced by `spawn-program'`?
//!
//! Shape (232.3 applied to the spawn handle): a protocol `:t::Spawnable` with one method;
//! `extend-type` it onto `Thread'` (and `Process'`); a fn typed `[h <- :t::Spawnable]` accepts a real
//! `Thread'` value from `spawn-program'` and dispatches the method on its concrete type.
//!
//! NOTE: renamed from `:wat::spawn::Spawned` → `:t::Spawnable` (stone host-parity-2 mints the
//! stdlib marker `:wat::spawn::Spawned` as a typesub/derive marker; the protocol here is a
//! test-local name to avoid the clash). Stays a valid arc-267 regression test (protocol bound
//! over the opaque `Thread'`; only the name changed).
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
(:wat::core::defprotocol :t::Spawnable
  (spawned-tag [self <- :t::Spawnable] -> :wat::core::String))

;; extend-type onto the BUILT-IN OPAQUES (the novel part — 232 only did user Records).
(:wat::core::extend-type :wat::kernel::Thread'  :t::Spawnable (spawned-tag [self] "thread"))
(:wat::core::extend-type :wat::kernel::Process' :t::Spawnable (spawned-tag [self] "process"))

;; A fn typed over the handle bound — accepts any extender, dispatches on the concrete handle type.
(:wat::core::defn :user::tag-of [h <- :t::Spawnable] -> :wat::core::String
  (:t::Spawnable/spawned-tag h))

;; Get a REAL Thread' from spawn-program' (thread tier) and pass it through the :t::Spawnable bound.
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
        .expect("startup should succeed (handle protocol :t::Spawnable extend-typed onto Thread'/Process')");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(&got, Value::String(s) if s.as_str() == "thread"),
        "expected \"thread\": a Thread' (built-in opaque) passed through a :t::Spawnable-typed param must \
         dispatch spawned-tag to the Thread' impl; got {got:?}"
    );
}

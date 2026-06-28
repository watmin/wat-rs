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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn handle_protocol_binds_and_dispatches_over_builtin_opaque() {
    // Wat source lives in the co-located fixture: probe_arc209_handle_protocol.wat
    let world = startup_beside(file!())
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

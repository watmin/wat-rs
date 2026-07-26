//! Arc 209 (host-parity leg, stone 2) — the `:wat::spawn::Spawned` handle marker.
//!
//! The uniform bound the host-agnostic defservice `Handle` field needs: the spawn handles
//! `Thread'<I,O>`/`Process'<I,O>` (and future remote handles) all `derive` it, so `Handle.handle`
//! can hold any of them. It is a typesub/`isa?` marker (Clojure's `derive` axis) — NO methods, NOT a
//! protocol. The stone adds, in `wat/spawn.wat`:
//!   (:wat::core::derive :wat::kernel::Thread'  :wat::spawn::Spawned)
//!   (:wat::core::derive :wat::kernel::Process' :wat::spawn::Spawned)
//! and retypes defservice's `Handle.handle` from `Thread'<Op,Reply>` to `:wat::spawn::Spawned`.
//!
//! This probe: a real `Thread'` from `spawn-program'` flows to a `:wat::spawn::Spawned`-typed param.
//! It exercises the STDLIB marker (not an inline declaration) — the derive edges live in spawn.wat.
//! Builds on the shipped `derive` verb (arc 237 follow-on) + parametric protocol bounds (arc 267 —
//! `Thread'<I,O>` is parametric, so the `assignable` head-arm carries the satisfaction).
//!
//! RED at HEAD: `wat/spawn.wat` does not yet derive `Thread'`→`:Spawned`, so `Thread'<i64,i64>` is
//! not a subtype of `:wat::spawn::Spawned` → the bound rejects it. GREEN once the stone adds the
//! derive edges.
//!
//! Run SERIALLY (spawns a thread):
//!   cargo test --release -p wat --test probe_arc209_spawned_marker -- --test-threads=1

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn thread_handle_derives_the_spawned_marker() {
    // Wat source lives in the co-located fixture: probe_arc209_spawned_marker.wat
    let got = call_beside_value(file!(), ":user::go")
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(99)),
        "expected 99: a Thread' (which derives :wat::spawn::Spawned in spawn.wat) must be accepted \
         where the :Spawned marker bound is required; got {got:?}"
    );
}

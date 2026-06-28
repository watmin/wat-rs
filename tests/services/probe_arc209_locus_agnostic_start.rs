//! Arc 209 locus-parity leg, stone 4a — the locus-agnostic `start [locus <- :Locus]`.
//!
//! C.3 shipped a THREAD-HARDCODED `start [state0]` (it bakes `(:wat::spawn::thread)` into the
//! `listener'` + `spawn-program'` calls, `wat/service.wat` start-body). Stone 4a makes `start` take
//! the locus as its first arg and route through the `:wat::spawn::Locus` protocol (arc 232) — so one
//! `start` serves any transport, dispatching the per-tier launch to the `extend-type :Locus` impl.
//! Thread is `extend-type :wat::spawn::ThreadOpts :Locus` (the shared-memory/capture strategy — the
//! `deftest'` model); process/remote join later as not-shared `extend-type`s, zero edit to `start`.
//!
//! THE GATE (thread-proven): the SAME counter as C.3, driven through the client face, but
//! `(:my::counter/start (:wat::spawn::thread) 0)` — start now takes a locus. The round-trip
//! (increment 5 → get → 5) proves the locus-agnostic start dispatches the thread launch correctly.
//!
//! RED at HEAD: C.3's `start` is arity-1 `[state0]` — `(counter/start (thread) 0)` is a 2-arg call
//! → arity mismatch; and `:wat::kernel::Locus` / the ThreadOpts `extend-type` don't exist. GREEN once
//! 4a ships the Locus protocol + thread impl + the locus-agnostic start codegen.
//!
//! Run: cargo test --release -p wat --test probe_arc209_locus_agnostic_start

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn locus_agnostic_start_dispatches_thread_launch() {
    // arc 291 4b-ii: State is now a defstruct; :durable mints ::Record; start takes ::Record.
    // Wat source lives in the co-located fixture: probe_arc209_locus_agnostic_start.wat
    let world = startup_beside(file!())
        .expect("startup should succeed (4a: start [locus <- :Locus] dispatches the thread launch via the Locus protocol)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5 driven through the locus-agnostic client face \
         (start (thread) 0 → connect → increment 5 → get); got {got:?}"
    );
}

//! Arc 259 S2c-ii-b — `spawn-program'` as the host-type `defclause` (FM-2-bis probe).
//!
//! The keystone: `spawn-program'` becomes a 2-arg `(host prog)` wat defclause
//! dispatching on the host record type (`ThreadOpts`/`ProcessOpts`), retiring the
//! 3-arg Rust intrinsic. Unblocked by S2c-ii.0 (class_fqdn dispatch); simplified by
//! S2c-ii-a (apply-loop purged → one thread clause, no overlap). Full design:
//! `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2c-ii.md`.
//!
//! ## Why this is RED at HEAD
//!
//! At HEAD `spawn-program'` is the 3-arg `(:tier env prog)` intrinsic; the 2-arg
//! `(spawn-program' (thread) prog)` form is an arity mismatch. Post-S2c-ii-b the
//! defclause dispatches on `(thread)`'s `ThreadOpts` type → spawns → 42.
//!
//! Run: `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_s2cii_b_defclause`
//!
//! WAT fixture: tests/kernel/probe_arc259_s2cii_b_defclause.wat (co-located sibling)

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_compute_i64() -> i64 {
    let result = call_beside_value(file!(), ":user::compute")
        .expect("compute (RED at HEAD: spawn-program' is the 3-arg intrinsic; 2-arg is arity-mismatch)");
    match result {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

/// LOAD-BEARING: the 2-arg `(spawn-program' (thread) <self-peer-prog>)` form
/// dispatches on the `ThreadOpts` host type and round-trips 42.
#[test]
fn s2cii_b_two_arg_host_dispatch() {
    assert_eq!(run_compute_i64(), 42, "(spawn-program' (thread) prog) host-dispatches + echoes 42");
}

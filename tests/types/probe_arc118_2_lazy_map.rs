//! Arc 118.2 — DISCONFIRMING PROBE: `:wat::core::map` is LAZY (does not force the whole input).
//!
//! **The wat source is the co-located sibling fixture** `probe_arc118_2_lazy_map.wat`, slurped via
//! `startup_beside(file!())` — the repo's test-fixture scheme (never inlined as a Rust string).
//! `startup_from_*` only LOADS; the map body runs when we `eval_in_frozen` the `(:my::compute)` call.
//!
//! RED at HEAD: eager `core::map` applies `boom` to every element at the map call → `boom(99)`
//! (div-by-zero) → eval Errs. GREEN at 118.2a (lazy): only `boom(1)` runs → returns `1`.
//!
//! `#[ignore]`'d: 118.2a is unbuilt AND 118.2 is BLOCKED on 293.4 (`Seqable` needs methods-as-accessors).

use wat::freeze::call_beside;

#[test]
fn lazy_core_map_does_not_force_late_elements() {
    let result = call_beside(file!(), ":my::compute");
    assert!(
        result.is_ok(),
        "core::map must be LAZY — pulling only the head must not force the late div-by-zero; got: {:?}",
        result.err()
    );
}

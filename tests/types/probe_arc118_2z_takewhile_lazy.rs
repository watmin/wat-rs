//! Arc 118.2-Z strike A — DISCONFIRMING PROBE: `:wat::core::take-while` exists AND is LAZY.
//!
//! The wat source is the co-located sibling fixture `probe_arc118_2z_takewhile_lazy.wat`, slurped
//! via `startup_beside(file!())` (the repo's test-fixture scheme — never inlined as a Rust string).
//! `startup_from_*` only LOADS; the take-while runs when we `eval_in_frozen` the `(:my::compute)` call.
//!
//! RED at HEAD: `:wat::core::take-while` is undefined → the fixture fails to load / resolve.
//! GREEN after strike A: take-while is a lazy defclause; `(< x 3)` stops at 5, the `boom(99)` cell of
//! the lazy `map` source is never realized, `into []` yields `[1 2]`.
//!
//! `#[ignore]`'d until strike A ships the lazy transformer family (`wat/seq.wat`).

use wat::freeze::call_beside_value;

#[test]
fn lazy_take_while_stops_before_forcing_late_boom() {
    let result = call_beside_value(file!(), ":my::compute");
    assert!(
        result.is_ok(),
        "take-while must be LAZY — stopping at the first false must not force the later boom(99); got: {:?}",
        result.err()
    );
}

//! Arc 278 Cache Stone 1 gate — the functional proof that the baked `:wat::cache::Lru`
//! primitive (`src/rust_deps/cache.rs` + `wat/cache.wat`, a fresh thread-owned bounded LRU
//! over the `lru` crate) round-trips new/put/get/len, that the capacity bound really evicts
//! (cap-2, three keys), that the evicted pair comes back as a NAMED `:wat::cache::Entry`
//! (key/value) rather than a positional tuple, and that `<K,V>` is genuinely generic over
//! hashable EDN (keyword keys, i64 values).
//!
//! Run: `cargo nextest run --release -E 'test(cache_lru)'`
//!
//! # Why this weighs the VERDICT and not `result.is_ok()`
//!
//! A `deftest` fn returns a `:wat::kernel::RunResult` VALUE — an assertion that fires is
//! captured into it, NOT raised as a Rust `RuntimeError`. So the old
//! `call_beside(...).is_ok()` idiom proved only that the fixture FROZE and the body RAN;
//! every `assert-eq` inside it could be falsified and the Rust test still passed. Verified
//! empirically on 2026-07-25 by mutating this fixture (and its `probe_arc278_sqlite_interop`
//! sibling, which had the same hole) to a deliberately wrong expectation — both stayed green.
//!
//! Arc 278 the vacuous-gate wall closed that at the type level: `call_beside` returns a
//! `DeftestOutcome` with no `is_ok()`, and `RunResult` is an ENUM (`:Passed` /
//! `:Failed[failure]`) rather than a struct with an ignorable `Option` slot. This gate no
//! longer needs a golden EDN encoding of RunResult's innards — the verdict IS the assertion.

use wat::freeze::call_beside;

#[test]
fn cache_lru() {
    call_beside(file!(), ":user::cache_lru").expect_passed(
        "cache_lru deftest must pass (new/put/get/len round-trip + cap-2 eviction returning \
         Some Entry{:key :a :value 1})",
    );
}

//! Arc 278 Cache Stone 1 gate — the functional proof that the baked `:wat::cache::Lru`
//! primitive (`src/rust_deps/cache.rs` + `wat/cache.wat`, a fresh thread-owned bounded LRU
//! over the `lru` crate) round-trips new/put/get/len, that the capacity bound really evicts
//! (cap-2, three keys), that the evicted pair comes back as a NAMED `:wat::cache::Entry`
//! (key/value) rather than a positional tuple, and that `<K,V>` is genuinely generic over
//! hashable EDN (keyword keys, i64 values).
//!
//! Run: `cargo nextest run --release -E 'test(cache_lru)'`
//!
//! # Why this weighs the RunResult and not `result.is_ok()`
//!
//! A `deftest` fn returns a `:wat::kernel::RunResult` VALUE — an assertion that fires is
//! captured into its `failure` slot, NOT raised as a Rust `RuntimeError`. So the neighbouring
//! `call_beside(...).is_ok()` idiom proves only that the fixture FROZE and the body RAN; every
//! `assert-eq` inside it can be falsified and the Rust test still passes. Verified empirically
//! on 2026-07-25 by mutating this fixture (and its `probe_arc278_sqlite_interop` sibling, which
//! has the same hole) to a deliberately wrong expectation — both stayed green. This gate
//! therefore asserts on the returned RunResult's STRUCTURE: `failure` must be `None`.

use wat::edn_shim::value_to_edn;
use wat::freeze::call_beside;

#[test]
fn cache_lru() {
    let result = call_beside(file!(), ":user::cache_lru")
        .unwrap_or_else(|e| panic!("cache_lru fixture must evaluate; got Err: {e:?}"));

    // A passing deftest is a RunResult whose only field — `failure`, field 0, which
    // `value_to_edn` renders positionally as `:field-0` — is None. Anything else (a fired
    // assert-eq, a panic) lands a `:wat::kernel::Failure` record in that slot.
    wat::assert_edn_eq!(
        wat_edn::write(&value_to_edn(&result)),
        include_str!("probe_arc278_cache_lru__pass.edn"),
        "cache_lru deftest must pass (new/put/get/len round-trip + cap-2 eviction returning \
         Some Entry{:key :a :value 1}); a non-None :failure slot IS the assertion that fired"
    );
}

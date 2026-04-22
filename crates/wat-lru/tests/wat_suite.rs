//! Arc 015 slice 3 — wat-lru self-test via `wat::test_suite!`.
//!
//! Syntactically identical to how a downstream consumer declares
//! `deps: [wat_lru]`. Integration tests in `tests/` see their own
//! crate under its published name (standard Cargo shape), so this
//! IS the consumer shape — wat-lru is its own first consumer.
//!
//! `cargo test -p wat-lru` invokes the generated `#[test] fn
//! wat_suite`, which runs `wat::test_runner::run_and_assert` against
//! every `.wat` file under `wat-tests/` with wat-lru's
//! `wat_sources()` + `register()` composed in. Test output
//! appears under the `wat_suite` test banner.

wat::test_suite! {
    path: "wat-tests",
    deps: [wat_lru],
}

//! Arc 015 + arc 018 — wat-lru self-test via `wat::test!` in
//! minimal form.
//!
//! `path:` omitted → defaults to `"wat-tests"`. Only `deps:
//! [wat_lru]` is explicit because wat-lru is its own first
//! consumer: integration tests in `tests/` see their own crate
//! under its published name (standard Cargo shape).
//!
//! `cargo test -p wat-lru` invokes the generated `#[test] fn
//! wat_suite`, which runs every `.wat` file under `wat-tests/`
//! with wat-lru's `wat_sources()` + `register()` composed in.

wat::test! {
    deps: [wat_lru],
}

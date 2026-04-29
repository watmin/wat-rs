//! Arc 091 slice 2 — wat-measure self-test via `wat::test!`.
//!
//! `cargo test -p wat-measure` invokes the generated `#[test] fn
//! wat_suite`, which runs every `.wat` file under `wat-tests/`
//! with wat-measure's `wat_sources()` + `register()` composed in.
//! `path:` defaults to `"wat-tests"`; only `deps: [wat_measure]` is
//! explicit because wat-measure is its own first consumer.

wat::test! {
    deps: [wat_telemetry, wat_measure],
}

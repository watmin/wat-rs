//! Arc 096 slice 1 — wat-telemetry self-test via `wat::test!`.
//!
//! `cargo test -p wat-telemetry` runs every `.wat` file under
//! `wat-tests/` with this crate's `wat_sources()` + `register()`
//! composed in. Same shape any downstream consumer uses when it
//! declares `deps: [wat_telemetry]`.

wat::test! {
    deps: [wat_telemetry],
}

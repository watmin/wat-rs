//! Arc 096 slice 2 — wat-telemetry-sqlite self-test via `wat::test!`.
//!
//! `cargo test -p wat-telemetry-sqlite` runs every `.wat` file
//! under `wat-tests/` with this crate's `wat_sources()` +
//! `register()` composed in, plus its two underlying deps —
//! wat-telemetry (for Service<E,G>) and wat-sqlite (for Db).

wat::test! {
    deps: [wat_telemetry, wat_sqlite, wat_telemetry_sqlite],
}

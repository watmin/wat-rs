//! Arc 083 slice 0 — wat-sqlite self-test via `wat::test! {}`.
//!
//! `cargo test -p wat-sqlite` invokes the generated test; the
//! runner discovers every `.wat` file under `wat-tests/` and
//! pulls wat-sqlite's `wat_sources()` / `register()` into scope.
//!
//! Slice 0 ships an empty source list; the test runs but discovers
//! nothing. Slices 1 + 2 add real wat-tests.

wat::test! {
    deps: [wat_sqlite],
}

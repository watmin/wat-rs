//! Arc 074 slice 2 — wat-holon-lru self-test via `wat::test! {}`.
//!
//! `cargo test -p wat-holon-lru` invokes the generated test;
//! the runner discovers every `.wat` file under `wat-tests/` with
//! both wat-lru AND wat-holon-lru's `wat_sources()` /
//! `register()` composed in.

wat::test! {
    deps: [wat_lru, wat_holon_lru],
}

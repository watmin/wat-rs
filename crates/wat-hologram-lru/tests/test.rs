//! Arc 074 slice 2 — wat-hologram-lru self-test via `wat::test! {}`.
//!
//! `cargo test -p wat-hologram-lru` invokes the generated test;
//! the runner discovers every `.wat` file under `wat-tests/` with
//! both wat-lru AND wat-hologram-lru's `wat_sources()` /
//! `register()` composed in.

wat::test! {
    deps: [wat_lru, wat_hologram_lru],
}

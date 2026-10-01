//! Arc 255 Stone 255.75 — "every probe runs" (the builder's ruling, E3): "A probe
//! asserts its own claim in wat. The floor runs every probe and requires exit 0.
//! A probe that must fail is a `.wat.bad` with a driven test naming its error."
//!
//! ONE `#[test]` per `wat-scripts/probes/**/*.wat` (never `.wat.bad`), generated
//! by `build.rs` into `OUT_DIR/every_probe_runs.rs` — so a red names the exact
//! probe, and nextest runs them in parallel (one process per probe). A probe
//! dropped anywhere under `wat-scripts/probes/` is picked up with NO edit to
//! this file or to `build.rs`: the generator walks the tree at build time and
//! `rerun-if-changed` covers every subdirectory, so adding/removing a `.wat`
//! file re-triggers generation on the next build.
//!
//! Each generated test runs the release binary (`CARGO_BIN_EXE_wat`) directly
//! against the probe's real path, stdin `/dev/null`, a 10s timeout (killed via
//! `kill -9` on expiry), and requires exit 0.
//!
//! Mutation-proven (255.75's SCORE): a probe temporarily made to raise turned
//! its own generated test red, by name; restoring the probe turned it green
//! again — nothing else moved.

include!(concat!(env!("OUT_DIR"), "/every_probe_runs.rs"));

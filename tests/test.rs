//! Arc 022 slice 2 — wat-rs eats its own dog food.
//!
//! `cargo test` runs the full `wat-tests/` suite natively via the
//! opinionated-defaults `wat::test! {}` expansion. No binary spawn,
//! no subprocess dance — integration test links against the wat
//! crate and invokes the library path directly.
//!
//! This is the same minimal shape consumer crates adopt (arc 018):
//! one `tests/test.rs`, one macro invocation, `path:` defaulted to
//! `"wat-tests"`, `loader:` defaulted to `"wat-tests"`. wat-rs
//! carries zero external wat deps, so `deps:` is empty.
//!
//! The CLI path stays covered by `tests/wat_test_cli.rs` — that
//! one spawns the built binary to verify the command-line surface.
//! This file tests the library surface.

wat::test! {}

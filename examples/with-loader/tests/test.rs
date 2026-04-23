//! Arc 017 + arc 018 — test-side minimal form. `wat::test! {}`
//! picks up the defaults: `path: "wat-tests"` + `loader:
//! "wat-tests"`. The suite runs `wat-tests/test_loader.wat`, which
//! `(:wat::load-file! "helpers.wat")`s its
//! sibling via the default ScopedLoader.
//!
//! Silent on success. Run with `--nocapture` for per-wat-test
//! output: `cargo test -p with-loader-example -- --nocapture`.

wat::test! {}

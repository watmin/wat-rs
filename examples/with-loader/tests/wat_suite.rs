//! Arc 017 slice 2 — `wat::test_suite! { ..., loader: "..." }` proof.
//! The suite loads `wat-tests/test_loader.wat`, which itself
//! `(:wat::core::load! :wat::load::file-path "helpers.wat")`s its
//! sibling — resolved via the ScopedLoader the `loader: "wat-tests"`
//! arg constructs.
//!
//! Silent on success. Run with `--nocapture` for per-wat-test output:
//! `cargo test -p with-loader-example -- --nocapture`.

wat::test_suite! {
    path: "wat-tests",
    loader: "wat-tests",
}


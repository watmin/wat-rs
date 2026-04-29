//! `wat` — the canonical batteries-included wat CLI binary.
//!
//! Thin wrapper around [`wat_cli::run`]: declares the workspace's
//! 5 `#[wat_dispatch]` extension crates as batteries and lets the
//! library do the rest. Anyone wanting their OWN CLI with a
//! different battery set authors their own binary calling
//! `wat_cli::run(&[...])` directly — see crate-level docs.

use std::process::ExitCode;

fn main() -> ExitCode {
    wat_cli::run(&[
        (wat_telemetry::register, wat_telemetry::wat_sources),
        (wat_sqlite::register, wat_sqlite::wat_sources),
        (wat_lru::register, wat_lru::wat_sources),
        (wat_holon_lru::register, wat_holon_lru::wat_sources),
        (
            wat_telemetry_sqlite::register,
            wat_telemetry_sqlite::wat_sources,
        ),
    ])
}

//! `wat` — the canonical batteries-included wat CLI binary.
//!
//! Thin wrapper around [`wat_cli::run`]: declares the workspace's
//! `#[wat_dispatch]` extension crates as batteries and lets the
//! library do the rest. Anyone wanting their OWN CLI with a
//! different battery set authors their own binary calling
//! `wat_cli::run(&[...])` directly — see crate-level docs.
//!
//! Arc 278 Cache Stone 5: the workspace's last `#[wat_dispatch]`
//! extension crates (`wat-lru`, `wat-holon-lru`) were annihilated —
//! their capability moved into core (`wat/cache.wat`). No extension
//! battery remains, so this is the substrate-only shape: an empty
//! slice. `wat_cli::run`'s `RustDepsBuilder::with_wat_rs_defaults`
//! seeds every `:wat::*` surface regardless of the battery list.

use std::process::ExitCode;

fn main() -> ExitCode {
    wat_cli::run(&[])
}

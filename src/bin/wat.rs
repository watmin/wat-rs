//! `wat` — the canonical batteries-included wat CLI binary.
//!
//! Thin wrapper around [`wat::distribution::run`]: declares the workspace's
//! `#[wat_dispatch]` extension crates as batteries and lets the
//! library do the rest. Anyone wanting their OWN CLI with a
//! different battery set authors their own binary calling
//! `wat::distribution::run(&[...])` directly — see `wat::distribution`'s
//! module docs.
//!
//! Arc 278 Cache Stone 5: the workspace's last `#[wat_dispatch]`
//! extension crates (`wat-lru`, `wat-holon-lru`) were annihilated —
//! their capability moved into core (`wat/cache.wat`). No extension
//! battery remains, so this is the substrate-only shape: an empty
//! slice. `wat::distribution::run`'s `RustDepsBuilder::with_wat_rs_defaults`
//! seeds every `:wat::*` surface regardless of the battery list.
//!
//! Arc 170: folded in from the sibling `wat-cli` crate — same binary,
//! same shape, now built from `wat`'s own `src/bin/`.

use std::process::ExitCode;

fn main() -> ExitCode {
    wat::distribution::run(&[])
}

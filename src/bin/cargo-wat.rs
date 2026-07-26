//! `cargo-wat` — run a wat program as a cargo subcommand: `cargo wat <file.wat>`.
//!
//! Cargo injects the subcommand name as argv[1]; we strip it via
//! [`wat::distribution::strip_cargo_subcommand`], then defer to
//! [`wat::distribution::run_with_args`] with the same battery set as the
//! canonical `wat` binary.
//!
//! # Cargo dispatch convention
//!
//! `cargo X ...args...` finds `cargo-X` on PATH and invokes it as:
//!
//! ```text
//! cargo-X X ...args...
//! ```
//!
//! The repeated subcommand name at argv[1] is cargo's convention.
//! Stripping it makes the resulting argv identical to a direct
//! `./cargo-wat <file.wat>` invocation, which `run_with_args` expects.

//! Arc 278 Cache Stone 5: the workspace's last `#[wat_dispatch]`
//! extension crates (`wat-lru`, `wat-holon-lru`) were annihilated —
//! their capability moved into core (`wat/cache.wat`). Same
//! substrate-only battery set (empty) as the canonical `wat` binary.
//!
//! Arc 170: folded in from the sibling `wat-cli` crate — same binary,
//! same shape, now built from `wat`'s own `src/bin/`. `cargo wat`
//! resolves because this binary is literally named `cargo-wat` (the
//! `[[bin]] name =` entry in `Cargo.toml`); that's the whole mechanism.

use std::process::ExitCode;

fn main() -> ExitCode {
    let argv = wat::distribution::strip_cargo_subcommand(std::env::args().collect(), "wat");
    wat::distribution::run_with_args(&[], argv)
}

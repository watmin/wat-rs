//! Integration test root for `src/comms/`.
//!
//! Tests organized to mirror src/ directory structure (convention started
//! 2026-05-19 with arc 214 Slice 3). The `mod.rs` root + sub-files pattern
//! is the idiomatic Rust way to group related integration tests:
//!
//!   - `tests/comms/mod.rs` is the crate root for the comms test binary
//!     (registered in Cargo.toml via a single `[[test]] name = "comms"
//!     path = "tests/comms/mod.rs"` entry)
//!   - `mod foundation; mod thread; mod process;` brings the sub-files
//!     in via standard Rust same-directory module resolution
//!   - all comms tests share one binary (one link; one `#[ctor]` init)
//!   - filter via `cargo test --test comms <path>::` — e.g.,
//!     `cargo test --test comms thread::` runs only thread-tier tests
//!
//! Future modules follow the same template:
//!
//!   - `tests/<module>/mod.rs` (crate root; `mod sub1; mod sub2;`)
//!   - `tests/<module>/{sub1,sub2}.rs` (actual tests)
//!   - one `[[test]]` Cargo.toml entry per module
//!
//! Cargo intentionally does NOT auto-discover subdirectory files as test
//! binaries — subdirs are reserved for helper modules (the well-known
//! `tests/common/mod.rs` idiom). The `[[test]]` per-module entry works
//! WITH that design; the cost is one Cargo.toml line per module group.

mod foundation;
mod process;
mod thread;

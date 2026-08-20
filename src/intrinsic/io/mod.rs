//! `:wat::io::` intrinsic registry entries — arc 255.1c, home #12.
//!
//! **`:wat::io::` IS a family** — unlike `:wat::kernel::`, whose own `mod.rs`
//! opens *"`:wat::kernel::` is not a family. It is a TIER"* (nine homes
//! braiding independent concerns, each with its own reason to change). This
//! namespace is ONE subject — bytes crossing the process boundary — asked
//! three ways:
//!
//! - [`reader`] — pull bytes IN: construct an `IOReader` (from bytes, a
//!   string, a file, or a raw fd) and read from it (`read`, `read-all`,
//!   `read-all-string`, `read-line`, `read-frame`, `rewind`).
//! - [`writer`] — push bytes OUT: the `IOWriter` mirror (`new`, `open-file`,
//!   `from-fd`, `to-bytes`, `to-string`, `write`, `write-all`,
//!   `write-string`, `print`, `println`, `writeln`, `flush`, `close`).
//! - `fs` — the filesystem-adjacent one-shots and the two RAII temp handles
//!   (not yet carved; stone 255.1c-io-fs, 6 rows).
//!
//! **The bodies do not live here.** All thirty arms across the family's
//! eventual three files delegate into `crate::io::` — one module, so the
//! "bodies do not live in this tier" claim `kernel/mod.rs` makes for its
//! nine homes holds for this family's three, for the same reason: carving a
//! verb into the registry changes which PATH reaches its handler (registry
//! lookup vs. a literal `match head { … }` arm in `runtime.rs`), never the
//! handler itself.

mod reader;
mod writer;

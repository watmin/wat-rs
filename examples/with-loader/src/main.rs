//! Arc 017 + arc 018 — reference binary for the minimal consumer
//! shape. No args to `wat::main!` — `wat/main.wat` is the implicit
//! entry, `"wat"` is the implicit loader root. The recursive
//! `(load!)` chain under `wat/` (main.wat → helper.wat →
//! deeper.wat) resolves through that default ScopedLoader.
//!
//! Run: `cargo run -p with-loader-example`. Expected stdout:
//! `hello, wat-loaded`.

wat::main! {}

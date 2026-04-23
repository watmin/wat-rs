//! Arc 017 slice 1 — reference binary for `wat::main!`'s `loader:`
//! option. The `source:` is the entry wat file; it `(:wat::core::load!
//! ...)`'s sibling files from the `wat/` directory via the
//! `ScopedLoader` the `loader: "wat"` argument constructs.
//!
//! This is the walkable shape downstream consumers with multi-file
//! wat trees (starting with the trading lab) will follow.
//!
//! Run: `cargo run -p with-loader-example`. Expected stdout:
//! `hello, wat-loaded`.

wat::main! {
    source: include_str!("program.wat"),
    loader: "wat",
}

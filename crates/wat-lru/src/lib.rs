//! `wat-lru` — `:rust::lru::LruCache<K,V>` surfaced into wat.
//!
//! Shipped 2026-04-21 as arc 013's proof that external wat crates
//! are a real mechanism, not theory. Before this arc, LRU lived
//! inside wat-rs's baked stdlib at `:wat::std::LocalCache<K,V>`
//! and `:wat::std::service::Cache`. Arc 013 slice 4b moved both
//! surfaces + the `#[wat_dispatch]`-driven Rust shim + the `lru`
//! Cargo dep into this sibling crate, under the new community
//! namespace `:wat::lru::*` — arc 036 promoted wat-lru from the
//! community tier (`:user::*`) to the first-party tier (`:wat::*`)
//! on the rule that workspace-member crates of wat-rs share the
//! repo, the author, and the release cadence. The dispatch
//! mechanism already permitted this (installed deps bypass the
//! reserved-prefix gate at stdlib-tier registration); arc 036 is
//! the convention amendment + rename sweep.
//!
//! # Using wat-lru from a Rust binary crate
//!
//! ```text
//! // Cargo.toml
//! [dependencies]
//! wat     = { path = "../wat-rs" }
//! wat-lru = { path = "../wat-rs/crates/wat-lru" }
//!
//! // main.rs
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_lru],
//! }
//!
//! // program.wat
//! (:wat::core::use! :rust::lru::LruCache)
//! (:wat::core::define (:user::main
//!                      (stdin  :wat::io::IOReader)
//!                      (stdout :wat::io::IOWriter)
//!                      (stderr :wat::io::IOWriter)
//!                      -> :())
//!   (:wat::core::let*
//!     (((cache :wat::lru::LocalCache<String,i64>)
//!       (:wat::lru::LocalCache::new 16))
//!      ((_ :()) (:wat::lru::LocalCache::put cache "k" 42)))
//!     ()))
//! ```
//!
//! The two-part external-crate contract per arc 013:
//! - [`wat_sources`] — wat source files (the two `.wat` files
//!   baked via `include_str!`).
//! - [`register`] — Rust shim dispatch + schemes + type decls
//!   (the `#[wat_dispatch]`-generated register fn for
//!   `:rust::lru::LruCache`).

pub mod shim;

/// wat source files this crate contributes. Returned in
/// registration order: `LocalCache.wat` (LocalCache wrapper) first,
/// `CacheService.wat` (multi-client CacheService on top of LocalCache)
/// second. `wat::main!` / `wat::Harness::from_source_with_deps*`
/// / `wat::compose_and_run` / `wat::test_suite!` consume this slice.
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[
        wat::WatSource {
            path: "wat-lru/lru/LocalCache.wat",
            source: include_str!("../wat/lru/LocalCache.wat"),
        },
        wat::WatSource {
            path: "wat-lru/lru/CacheService.wat",
            source: include_str!("../wat/lru/CacheService.wat"),
        },
    ];
    FILES
}

/// Registrar for `:rust::lru::LruCache<K,V>`. Forwards to the
/// `#[wat_dispatch]`-generated register fn. `wat::main!` /
/// Harness / `wat::compose_and_run` call this to wire the Rust
/// shim into the process's `rust_deps::RustDepsRegistry`.
pub fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    shim::register(builder);
}

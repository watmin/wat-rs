//! `wat-holon-lru` — `:wat::holon::HologramLRU` exposed to wat as
//! pure-wat composition over `:wat::holon::Hologram` (substrate) and
//! `:wat::lru::LocalCache` (wat-lru).
//!
//! Arc 074 slice 2. The bounded sibling of `:wat::holon::Hologram` —
//! a coordinate-cell store with cosine readout (slice 1, in
//! wat-rs core) augmented with global LRU eviction. When the LRU
//! evicts a key, the corresponding Hologram cell entry is dropped.
//!
//! # Why this crate has no Rust code beyond the registrar
//!
//! HologramLRU's logic is a wat composition: `Hologram/put` +
//! `LocalCache::put` (now eviction-aware after the slice-2 prep
//! commit), with `Hologram/find-best` + filter on read. The substrate
//! primitives needed (`Hologram/find-best`, `Hologram/remove-at-index`,
//! `Hologram/pos-to-idx`) all live in core. wat-lru's
//! `LocalCache::put` returns `Option<(K, V)>` for evicted entries.
//!
//! The crate ships ONE wat source file — `wat/holon/HologramLRU.wat` —
//! plus its tests. The Rust side is just the registrar that exposes
//! the wat source to consumers.
//!
//! # Using from a Rust binary crate
//!
//! ```text
//! // Cargo.toml
//! [dependencies]
//! wat           = { path = "../wat-rs" }
//! wat-lru       = { path = "../wat-rs/crates/wat-lru" }
//! wat-holon-lru = { path = "../wat-rs/crates/wat-holon-lru" }
//!
//! // main.rs
//! wat::main! {
//!     source: include_str!("program.wat"),
//!     deps: [wat_lru, wat_holon_lru],
//! }
//! ```
//!
//! The crate has no Rust shim — `register()` is a no-op. The
//! external-crate contract uses `wat_sources()` only.

/// wat source files this crate contributes. Returned in registration
/// order: `HologramLRU.wat` (the bounded sibling of `Hologram`).
/// Consumers who want HologramLRU pass `[wat_lru, wat_holon_lru]`
/// (in that order — wat-lru registers `LocalCache` first, then
/// HologramLRU.wat layers on top).
pub fn wat_sources() -> &'static [wat::WatSource] {
    static FILES: &[wat::WatSource] = &[wat::WatSource {
        path: "wat-holon-lru/holon/HologramLRU.wat",
        source: include_str!("../wat/holon/HologramLRU.wat"),
    }];
    FILES
}

/// Registrar for wat-holon-lru. No-op: this crate ships pure wat;
/// no Rust shim. Present so the external-crate contract reads
/// uniformly with crates that DO have Rust code (wat-lru).
pub fn register(_builder: &mut wat::rust_deps::RustDepsBuilder) {
    // No-op: HologramLRU is wat-stdlib-only.
}

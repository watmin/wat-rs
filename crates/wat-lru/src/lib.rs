//! `wat-lru` — `:rust::lru::LruCache<K,V>` surfaced into wat.
//!
//! Skeleton at arc 013 slice 1. The substrate pieces land in later
//! slices:
//!
//! - **slice 2** lifts `wat::stdlib::StdlibFile` visibility to `pub`
//!   so external crates can return it.
//! - **slice 3** lands `wat::main!` + `wat::compose_and_run` —
//!   consumers compose this crate's `stdlib_sources()` with the
//!   baked stdlib and their own program.
//! - **slice 4** moves `wat/std/LocalCache.wat` + the
//!   `#[wat_dispatch] impl lru::LruCache<K,V>` shim + the `lru`
//!   Cargo dep here. Paths repath from `:wat::std::LocalCache` to
//!   `:user::wat::std::lru::LocalCache`, and the `:user::wat::std::
//!   service::Cache` service-program wrapper becomes
//!   `:user::wat::std::lru::CacheService`.
//! - **slice 5** proves the full user shape via
//!   `examples/with-lru/`.
//!
//! This crate's contract after slice 4 will be `pub fn
//! stdlib_sources() -> &'static [wat::stdlib::StdlibFile]` — the
//! same shape every future external wat crate exposes.

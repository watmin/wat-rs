//! `:rust::lru::LruCache<K,V>` shim — externalized from wat-rs in
//! arc 013 slice 4b.
//!
//! `#[wat_dispatch]` annotates a Rust `impl` block; the macro
//! generates dispatch, scheme, and registration code. Behavior is
//! identical to the pre-externalization version in wat-rs:
//! scope-safe via `ThreadOwnedCell<WatLruCache>` thread-id guard,
//! zero Mutex. The macro codegen uses `::wat::*` absolute paths so
//! this compiles from an external crate without path rewriting.
//!
//! # Why a newtype
//!
//! `#[wat_dispatch]` annotates a Rust `impl` block. We can't
//! annotate the upstream `lru::LruCache<K,V>` directly (orphan rule,
//! plus generic handling), so we wrap it in `WatLruCache` with
//! monomorphic `LruCache<Value, Value>` storage — Value: Hash + Eq
//! (arc 216 Stone 216.5a). The wat-level `<K,V>` type parameters are
//! phantom (declared via the attribute's `type_params = "K,V"`) and
//! enforced by the type checker; the runtime transports any `Value`.
//!
//! Stone 216.5d — `hashmap_key` deleted. LRU storage changed from
//! `LruCache<String, (Value, Value)>` to `LruCache<Value, Value>`.
//! The canonical-string key crutch is gone; `Value: Hash + Eq` is
//! the contract. `value_is_hashable` guard replaces the `hashmap_key`
//! panic path for opaque-handle values.

use lru::LruCache;
use std::num::NonZeroUsize;

use wat::rust_deps::RustDepsBuilder;
use wat::runtime::{value_is_hashable, Value};

use wat_macros::wat_dispatch;

/// Newtype wrapper around `lru::LruCache<Value, Value>`.
/// The wat type checker sees this as `:rust::lru::LruCache<K,V>` with
/// phantom K,V (see the `type_params` attribute below).
///
/// Stone 216.5d — storage is `LruCache<Value, Value>`. `Value: Hash + Eq`
/// (arc 216.5a) is the equality contract. `value_is_hashable` guards
/// opaque-handle keys before `push`/`get` so they produce panics with
/// clear messages instead of hitting `unreachable!()` in `impl Hash for Value`.
#[allow(clippy::mutable_key_type)]
pub struct WatLruCache {
    inner: LruCache<Value, Value>,
}

#[wat_dispatch(
    path = ":rust::lru::LruCache",
    scope = "thread_owned",
    type_params = "K,V"
)]
#[allow(clippy::mutable_key_type)]
impl WatLruCache {
    /// `:rust::lru::LruCache::new capacity` — capacity must be positive.
    /// The returned value is a `ThreadOwnedCell<WatLruCache>` wrapped
    /// in `Value::RustOpaque`; the cell binds to the calling thread.
    pub fn new(capacity: i64) -> Self {
        let cap_usize = if capacity > 0 {
            capacity as usize
        } else {
            // Capacity must be positive (lru::LruCache requires NonZero).
            // The macro doesn't yet marshal method-internal errors back
            // to wat as RuntimeError; until it does, invalid input
            // surfaces as a panic. Startup integration tests catch the
            // message before users see it in production.
            panic!(
                ":rust::lru::LruCache::new: capacity must be positive; got {}",
                capacity
            );
        };
        WatLruCache {
            inner: LruCache::new(
                NonZeroUsize::new(cap_usize).expect("cap_usize > 0 checked above"),
            ),
        }
    }

    /// `:rust::lru::LruCache::put cache k v` — insert or update. LRU
    /// evicts the least-recently-used entry if insertion pushed past
    /// capacity. Stone 216.5d — key is `Value` directly via `Value: Hash + Eq`.
    /// Opaque-handle values (fn, channel, thread handle) are not hashable;
    /// `value_is_hashable` guards them before `push` so a clear panic is
    /// emitted instead of hitting `unreachable!()`.
    ///
    /// Returns `Some((evicted_k, evicted_v))` if insertion pushed past
    /// capacity, `None` otherwise. Most callers ignore the return; bound
    /// caches that maintain correlated state (HologramCache's per-cell
    /// hologram store) consume it to drop the matching entry.
    pub fn put(&mut self, k: Value, v: Value) -> Option<(Value, Value)> {
        if !value_is_hashable(&k) {
            panic!(
                ":rust::lru::LruCache::put: key must be a hashable value \
                 (primitive or HolonAST); got {}",
                k.type_name()
            );
        }
        // Use `push` (returns Option<(K,V)>) rather than `put` (returns
        // Option<V>) because we want eviction visibility, not just
        // overwrite visibility. push returns Some on either an
        // overwrite of an existing key OR a capacity-driven eviction.
        self.inner.push(k, v)
    }

    /// `:rust::lru::LruCache::get cache k` — returns `:Option<V>`. Hit
    /// bumps the entry to MRU. Stone 216.5d — key lookup via `Value: Hash + Eq`.
    pub fn get(&mut self, k: Value) -> Option<Value> {
        if !value_is_hashable(&k) {
            panic!(
                ":rust::lru::LruCache::get: key must be a hashable value \
                 (primitive or HolonAST); got {}",
                k.type_name()
            );
        }
        self.inner.get(&k).cloned()
    }

    /// `:rust::lru::LruCache::len cache` — current entry count.
    /// Read-only; does not affect LRU order. Used by telemetry to
    /// emit cache-size metrics (lab umbrella 059's L1/L2 cache
    /// service programs flush this through rundb on a rate gate).
    pub fn len(&self) -> i64 {
        self.inner.len() as i64
    }

    /// `true` iff the cache holds no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Registrar for `:rust::lru::LruCache`. Forwards to the
/// macro-generated register fn. Called by wat-lru's `pub fn register()`
/// at the crate root, which user binaries wire via `wat::main!`.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatLruCache::register(builder);
}

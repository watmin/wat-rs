//! `:rust::lru::LruCache<K,V>` — regenerated via `#[wat_dispatch]`.
//!
//! Task #195 replaced the hand-written shim with a macro-annotated
//! newtype `WatLruCache`. Behavior is identical to the hand-written
//! version (scope-safe via `ThreadOwnedCell<WatLruCache>` thread-id
//! guard, zero Mutex) — the macro generates the same dispatch,
//! scheme, and registration code the hand-written version had.
//!
//! # Why a newtype
//!
//! `#[wat_dispatch]` annotates a Rust `impl` block. We can't annotate
//! the upstream `lru::LruCache<K,V>` directly (orphan rule + generic
//! handling), so we wrap it in `WatLruCache` with monomorphic
//! `LruCache<String, Value>` storage — matching the hand-written
//! convention of canonical-string keys. The wat-level `<K,V>` type
//! parameters are phantom (declared via the attribute's
//! `type_params = "K,V"`) and enforced by the type checker; the
//! runtime transports any `Value`.

use lru::LruCache;
use std::num::NonZeroUsize;

use crate::runtime::{hashmap_key, Value};
use crate::rust_deps::RustDepsBuilder;

use wat_macros::wat_dispatch;

/// Newtype wrapper around `lru::LruCache<String, Value>`. The wat
/// type checker sees this as `:rust::lru::LruCache<K,V>` with phantom
/// K,V (see the `type_params` attribute below).
pub struct WatLruCache {
    inner: LruCache<String, Value>,
}

#[wat_dispatch(
    path = ":rust::lru::LruCache",
    scope = "thread_owned",
    type_params = "K,V"
)]
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
    /// evicts the least-recently-used entry if insertion pushes past
    /// capacity. Key is canonicalized via `hashmap_key` so
    /// heterogeneous types don't collide.
    pub fn put(&mut self, k: Value, v: Value) {
        // Non-primitive keys (HolonAST, Vec, handle values, …) panic:
        // HashMap-style canonicalization only covers the primitive key
        // domain. Same rationale as new() — errors-as-values round-trip
        // lands when the macro's return-type marshaling supports it.
        let key = hashmap_key(":rust::lru::LruCache::put", &k).unwrap_or_else(|_| {
            panic!(
                ":rust::lru::LruCache::put: key must be a primitive (got {})",
                k.type_name()
            )
        });
        self.inner.put(key, v);
    }

    /// `:rust::lru::LruCache::get cache k` — returns `:Option<V>`. Hit
    /// bumps the entry to MRU. Key constraint matches put().
    pub fn get(&mut self, k: Value) -> Option<Value> {
        let key = hashmap_key(":rust::lru::LruCache::get", &k).unwrap_or_else(|_| {
            panic!(
                ":rust::lru::LruCache::get: key must be a primitive (got {})",
                k.type_name()
            )
        });
        self.inner.get(&key).cloned()
    }
}

/// Entry point for wat-rs's default registry. Forwards to the
/// macro-generated register fn.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatLruCache::register(builder);
}

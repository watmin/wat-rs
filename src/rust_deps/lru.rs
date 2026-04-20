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
            // The macro's dispatch fn doesn't check this — it's just a
            // plain i64. Upstream wat error surfaces via a panic if we
            // hit NonZeroUsize::new(0); the hand-written version had
            // a pre-check that returned RuntimeError. With the macro
            // path we move the check into new()'s body.
            //
            // TODO(194 or later): teach the macro to emit optional
            // pre-dispatch guards (e.g. `where capacity > 0`) for
            // this kind of invariant. For now: panic with a clear
            // message — startup tests catch it.
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
        let key = match hashmap_key(":rust::lru::LruCache::put", &k) {
            Ok(key) => key,
            Err(_) => {
                // Same convention as new(): invalid keys panic from
                // inside the method body. TODO: threading RuntimeError
                // back through the macro dispatch path is scope for
                // a later task.
                panic!(
                    ":rust::lru::LruCache::put: key must be a primitive (got {})",
                    k.type_name()
                );
            }
        };
        self.inner.put(key, v);
    }

    /// `:rust::lru::LruCache::get cache k` — returns `:Option<V>`. Hit
    /// bumps the entry to MRU.
    pub fn get(&mut self, k: Value) -> Option<Value> {
        let key = match hashmap_key(":rust::lru::LruCache::get", &k) {
            Ok(key) => key,
            Err(_) => {
                panic!(
                    ":rust::lru::LruCache::get: key must be a primitive (got {})",
                    k.type_name()
                );
            }
        };
        self.inner.get(&key).cloned()
    }
}

/// Entry point for wat-rs's default registry. Forwards to the
/// macro-generated register fn.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatLruCache::register(builder);
}

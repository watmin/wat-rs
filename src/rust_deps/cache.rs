//! `:rust::cache::Lru` — arc 278 Cache Stone 1: a FRESH, thread-owned bounded
//! LRU cache, core's SECOND default `:rust::` shim (registered from
//! `with_wat_rs_defaults`, `src/rust_deps/mod.rs`, beside `sqlite`).
//!
//! Study-only oracle: `crates/wat-lru/src/shim.rs` (`:rust::lru::LruCache`) —
//! the distributions-of-wat experiment that proved the shape. This is NOT a
//! copy of it: the semantics were re-authored here at the final
//! `:rust::cache::Lru` path, and the wat surface above it (`wat/cache.wat`)
//! hands the evicted pair back as a NAMED `:wat::cache::Entry` record rather
//! than the oracle's positional tuple. The crate stays intact until Stone 5.
//!
//! # Why a newtype
//!
//! `#[wat_dispatch]` annotates a Rust `impl` block, so the upstream
//! `lru::LruCache<K,V>` can't be annotated directly (orphan rule + generics).
//! [`WatCacheLru`] wraps a MONOMORPHIC `LruCache<Value, Value>`: `Value: Hash +
//! Eq` is the storage contract, and the wat-level `<K,V>` are PHANTOM —
//! declared via the attribute's `type_params = "K,V"` and enforced by the type
//! checker, while the runtime transports any hashable `Value`. This is what
//! keeps the primitive genuinely generic (`<keyword,i64>`, `<String,i64>`,
//! `<HolonAST,nil>` all ride the same storage) instead of narrowed to one
//! concrete key/value pair.
//!
//! # Opaque + thread-owned
//!
//! `lru::LruCache` is mutable state, so the handle is scope-guarded by a
//! [`ThreadOwnedCell`] (zero Mutex — the guard is structural, a thread-id
//! check, not a contended lock). `scope = "thread_owned"` on the attribute
//! makes the macro wrap the `Self` return in that cell before opaquing and
//! route every `&self`/`&mut self` method through `with_ref`/`with_mut`.
//!
//! # Failure surface (INHERITED from the oracle — see the Stone 1 report)
//!
//! Two guards `panic!` rather than returning a value: a non-positive
//! `capacity` (the backing `LruCache` requires a `NonZeroUsize`) and a
//! non-hashable key (an opaque handle — `impl Hash for Value` is
//! `unreachable!()` there, so the guard turns a substrate `unreachable!` into
//! a legible message). Unlike sqlite's fallible verbs, these are the two
//! *programming-error* inputs, and the checker already rejects an opaque-typed
//! key at most call sites.
//!
//! **The reason Stone 1 gave for deferring the conversion has EXPIRED, and the
//! decision is OPEN — it is tracked, not promised.** Stone 1's brief
//! (`BRIEF-cache-stone-1-primitive.md`) surfaced these panics as a question and
//! left them to "a later stone", on the ground that the dispatch macro could
//! not yet marshal a method-internal error back to wat. That is no longer true
//! at this HEAD: `#[wat_dispatch]` marshals `Result<T, E>` natively via the
//! blanket `ToWat`/`FromWat` impls, INCLUDING `Result<Self, E>` for a
//! constructor like `Lru::new` — see `src/rust_deps/sqlite.rs`'s "Errors-as-values
//! — the exact mechanism". So the conversion is now MECHANICALLY available and
//! what remains is a genuine design call: does the no-hidden-failures law reach
//! a programming-error input, or stop at a fallible one? Converting changes a
//! SHIPPED public surface (`:wat::cache::Lru::new` would return a Result every
//! caller must match) and must move `wat/cache.wat` in the same breath.
//!
//! Tracked as a decision row in
//! `docs/arc/2026/06/278-rules-engine/NEXT-STRIKES-theater-hunt.md`
//! ("exigere — the cache panic conversion"). Do not re-defer it in prose here;
//! the row is the only honest home for it.

use lru::LruCache;
use std::num::NonZeroUsize;

use wat_macros::wat_dispatch;

use crate::rust_deps::RustDepsBuilder;
use crate::runtime::{value_is_hashable, Value};

/// `:rust::cache::Lru<K,V>` — a bounded LRU over EDN values. Storage is
/// `LruCache<Value, Value>`; `K`/`V` live only in the type checker (see the
/// module doc's "Why a newtype").
#[allow(clippy::mutable_key_type)]
pub struct WatCacheLru {
    inner: LruCache<Value, Value>,
}

#[wat_dispatch(path = ":rust::cache::Lru", scope = "thread_owned", type_params = "K,V")]
#[allow(clippy::mutable_key_type)]
impl WatCacheLru {
    /// `:rust::cache::Lru::new capacity` — a cache bounded at `capacity`
    /// entries. The returned value is a `ThreadOwnedCell<WatCacheLru>` inside a
    /// `Value::RustOpaque`; the cell binds to the calling thread.
    ///
    /// `capacity <= 0` panics — the backing `LruCache` takes a `NonZeroUsize`
    /// and the dispatch macro cannot yet marshal a method-internal error back
    /// to wat as a `RuntimeError`. See the module doc's failure-surface note.
    pub fn new(capacity: i64) -> Self {
        if capacity <= 0 {
            panic!(":rust::cache::Lru::new: capacity must be positive; got {capacity}");
        }
        let cap = NonZeroUsize::new(capacity as usize).expect("capacity > 0 checked above");
        WatCacheLru {
            inner: LruCache::new(cap),
        }
    }

    /// `:rust::cache::Lru::put cache k v` — insert or update, bumping `k` to
    /// MRU. Returns `Some((k, v))` for the pair DISPLACED by this insert —
    /// either the capacity-driven eviction of the least-recently-used entry, or
    /// the previous binding when `k` was already present — and `None` when the
    /// insert displaced nothing.
    ///
    /// `push` (not `put`) is the backing call precisely because it returns the
    /// displaced `(K, V)` pair, not just the overwritten value: a composite
    /// cache that keeps correlated state beside the LRU (Stone 3's
    /// `HolographicLru`, whose hologram store must drop the evicted key too)
    /// needs the KEY back, not only the value.
    ///
    /// A non-hashable key (an opaque handle) panics — see the module doc.
    pub fn put(&mut self, k: Value, v: Value) -> Option<(Value, Value)> {
        if !value_is_hashable(&k) {
            panic!(
                ":rust::cache::Lru::put: key must be a hashable value; got {}",
                k.type_name()
            );
        }
        self.inner.push(k, v)
    }

    /// `:rust::cache::Lru::get cache k` — `Some(v)` on a hit (which bumps `k`
    /// to MRU), `None` on a miss. A non-hashable key panics — see the module doc.
    pub fn get(&mut self, k: Value) -> Option<Value> {
        if !value_is_hashable(&k) {
            panic!(
                ":rust::cache::Lru::get: key must be a hashable value; got {}",
                k.type_name()
            );
        }
        self.inner.get(&k).cloned()
    }

    /// `:rust::cache::Lru::len cache` — current entry count (never above
    /// capacity). Read-only: does NOT touch LRU order.
    pub fn len(&self) -> i64 {
        self.inner.len() as i64
    }

    /// `:rust::cache::Lru::is_empty cache` — `true` iff the cache holds no
    /// entries. Read-only; does not touch LRU order.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Registrar for `:rust::cache::Lru`. Forwards to the macro-generated register
/// fn; called from `RustDepsBuilder::with_wat_rs_defaults` (`src/rust_deps/mod.rs`)
/// beside `sqlite::register` — the cache surface is BAKED (`wat/cache.wat` in
/// `STDLIB_FILES`), so it must resolve with no consumer-crate registration.
pub fn register(builder: &mut RustDepsBuilder) {
    __wat_dispatch_WatCacheLru::register(builder);
}

//! `:rust::cache::Lru` — arc 278 Cache Stone 1: a FRESH, thread-owned bounded
//! LRU cache, core's SECOND default `:rust::` shim (registered from
//! `with_wat_rs_defaults`, `src/rust_deps/mod.rs`, beside `sqlite`).
//!
//! Study-only oracle (⚠ **GONE — Stone 5 annihilated it**): `crates/wat-lru/src/shim.rs`
//! (`:rust::lru::LruCache`) — the distributions-of-wat experiment that proved
//! the shape. This is NOT a copy of it: the semantics were re-authored here at
//! the final `:rust::cache::Lru` path, and the wat surface above it
//! (`wat/cache.wat`) hands the evicted pair back as a NAMED
//! `:wat::cache::Entry` record rather than the oracle's positional tuple.
//!
//! This paragraph used to end "the crate stays intact until Stone 5". Stone 5
//! LANDED — `crates/` holds no `wat-lru`, and the path above resolves to
//! nothing. The citation is kept because the PROVENANCE is still the honest
//! answer to "where did this shape come from"; it is marked so no reader burns
//! time hunting a deleted file. Corrected 2026-08-25.
//!
//! ⚠ **"Oracle" here is a STUDY oracle — a prior implementation read for shape
//! — and has nothing to do with rete's `$oracle`**, which is a live
//! differential reference under `wat/rete/oracle/`. The two senses were
//! conflated once already: a reader grepped for the rete convention, found
//! nothing, and concluded this file's citation was fabricated.
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
//! # Failure surface
//!
//! `new` returns `Result<Self, (i64, String, String)>` — sqlite's `RawFault`
//! shape, `(code, diagnostic, message)`, so ONE raw-fault shape reads across
//! every `:rust::` shim. A non-positive `capacity` is an `Err` value, never a
//! panic. ⚠ `code` is `0`: unlike sqlite there is no external result-code
//! space here — `new` has exactly one failure mode — and the column is carried
//! rather than dropped so this is not a second, cache-only tuple arity with
//! its own lift (precedent for `0` = "not an external error code":
//! `sqlite.rs::param_to_tosql`). The type is spelled as the literal tuple, not
//! a `RawFault` alias, because the macro's `rust_type_to_type_expr_tokens`
//! matches the UNRESOLVED type and an alias name is not in its known-type list
//! — every `sqlite.rs` method spells it out for the same reason.
//!
//! `put`/`get` still `panic!` on a non-hashable key
//! (an opaque handle — `impl Hash for Value` is `unreachable!()` there, so the
//! guard turns a substrate `unreachable!` into a legible message).
//!
//! **`new`'s conversion landed 2026-09-22** (excursus 003 stone A, curing
//! the-little-wat F-084). The mandate is recorded in
//! `docs/excursus/2026/09/003-the-little-wat-findings/DESIGN-stone-A-lru-new-returns-a-result.md`
//! and it SUPERSEDED the arc-109 note's axis: the line is not
//! total-vs-partial and not whose-fault-is-the-input — **a refusal must arrive
//! as a wat value.** `:wat::i64::/` is annotated `@Totality Partial` (a
//! divide-by-zero is a caller bug by any reading) and still refuses INSIDE the
//! language, with the user's span; a `panic!` here gave a wat program a Rust
//! backtrace note, the internal `:rust::` name, and no span at all.
//!
//! The mechanism is `src/rust_deps/sqlite.rs`'s "Errors-as-values — the exact
//! mechanism": `#[wat_dispatch]` marshals `Result<T, E>` natively via the
//! blanket `ToWat`/`FromWat` impls, INCLUDING `Result<Self, E>` for a
//! constructor, through `emit_return_marshal`'s `result_ok_is_self` arm. Zero
//! macro changes were needed.
//!
//! ⛔ **`put`/`get` are NOT converted, and that is a ruling, not an oversight.**
//! The arc-109 NOTE rules LEAVE for both and warns explicitly against
//! converting all three for symmetry; the builder's totality framing weakens
//! that defence but does not overturn it. Re-opening it is a separate ruling.
//!
//! Tracked as a NOTE in arc 109, which owns `src/rust_deps/`:
//! `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`
//! — RULED ON THE MERITS (convert `Lru::new`, whose capacity crosses a
//! serialization boundary; LEAVE `put`/`get`, whose key is a caller bug), and
//! the `Lru::new` half has now SHIPPED. Do not re-open it in prose here; that
//! note is the only honest home for it.

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
    /// `:rust::cache::Lru/new capacity` — a cache bounded at `capacity`
    /// entries. The returned value is a `ThreadOwnedCell<WatCacheLru>` inside a
    /// `Value::RustOpaque`; the cell binds to the calling thread.
    ///
    /// `capacity <= 0` is an `Err` `(code, diagnostic, message)` tuple, never a panic — the backing
    /// `LruCache` takes a `NonZeroUsize`, and a wat caller must be able to see
    /// that refusal as a wat value carrying its own span. See the module doc's
    /// failure-surface note for the mandate.
    ///
    /// ⭐ The `diagnostic` column names the **user-facing** verb
    /// `:wat::cache::Lru/new`, never this internal `:rust::` shim: naming the
    /// shim is half of what the-little-wat F-084 reports. `wat/cache.wat`'s
    /// `:wat::cache::Lru/new` lifts this tuple into `:wat::cache::Fault`
    /// verbatim — it does not re-word it — so this string IS what a wat
    /// program reads.
    pub fn new(capacity: i64) -> Result<Self, (i64, String, String)> {
        if capacity <= 0 {
            return Err((
                0,
                ":wat::cache::Lru/new".to_string(),
                format!("capacity must be positive; got {capacity}"),
            ));
        }
        let cap = NonZeroUsize::new(capacity as usize).expect("capacity > 0 checked above");
        Ok(WatCacheLru {
            inner: LruCache::new(cap),
        })
    }

    /// `:rust::cache::Lru/put cache k v` — insert or update, bumping `k` to
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
                ":rust::cache::Lru/put: key must be a hashable value; got {}",
                k.type_name()
            );
        }
        self.inner.push(k, v)
    }

    /// `:rust::cache::Lru/get cache k` — `Some(v)` on a hit (which bumps `k`
    /// to MRU), `None` on a miss. A non-hashable key panics — see the module doc.
    pub fn get(&mut self, k: Value) -> Option<Value> {
        if !value_is_hashable(&k) {
            panic!(
                ":rust::cache::Lru/get: key must be a hashable value; got {}",
                k.type_name()
            );
        }
        self.inner.get(&k).cloned()
    }

    /// `:rust::cache::Lru/len cache` — current entry count (never above
    /// capacity). Read-only: does NOT touch LRU order.
    pub fn len(&self) -> i64 {
        self.inner.len() as i64
    }

    /// `:rust::cache::Lru/is_empty cache` — `true` iff the cache holds no
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

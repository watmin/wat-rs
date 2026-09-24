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
//! # Failure surface — no guard here panics
//!
//! **A refusal arrives as a wat value.** Every guard in this file answers with a
//! value the calling program can see; none of them `panic!`s. (One route still
//! reaches a panic PAST the guard — the shallow-guard ⚠ below.)
//!
//! | verb | refusal | answer |
//! |---|---|---|
//! | `new` | `capacity <= 0` | `Err((code, diagnostic, message))` |
//! | `put` | non-hashable key | `Err((code, diagnostic, message))` |
//! | `get` | non-hashable key | `None` — a miss |
//!
//! The `Err` column is sqlite's `RawFault` shape, `(code, diagnostic, message)`,
//! so ONE raw-fault shape reads across every `:rust::` shim; `wat/cache.wat`
//! lifts it into `:wat::cache::Fault`. `diagnostic` names the **user-facing**
//! verb (`:wat::cache::Lru/new`, `:wat::cache::Lru/put`), never this internal
//! `:rust::` shim. ⚠ `code` is `0`: unlike sqlite there is no external
//! result-code space here, and the column is carried rather than dropped so this
//! is not a second, cache-only tuple arity with its own lift (precedent for `0`
//! = "not an external error code": `sqlite.rs::param_to_tosql`). The type is
//! spelled as the literal tuple, not a `RawFault` alias, because the macro's
//! `rust_type_to_type_expr_tokens` matches the UNRESOLVED type and an alias name
//! is not in its known-type list — every `sqlite.rs` method spells it out for
//! the same reason.
//!
//! `get` needs no error channel: a key `put` refuses to store cannot be present,
//! so "absent" is the true, total answer — the same one `HashMap`'s
//! `contains-key?` gives an unhashable key (`src/collection/eval.rs`,
//! `hashmap_contains_key_q_inner`: *"never inserted"*). The `put`/`get` guards
//! themselves must stay: `impl Hash for Value` is `unreachable!()` for an opaque
//! handle, so hashing the key unguarded would panic in the hasher instead.
//!
//! **The guard is DEEP** (excursus 003 stone E, 2026-09-23). `value_is_hashable`
//! recurses into exactly the variants `impl Hash for Value` recurses into, so a
//! hashable CONTAINER holding an opaque handle — `(Option.Some <an Lru handle>)`
//! as the key — is refused here too, not handed to the hasher's `unreachable!()`.
//! Before stone E it was shallow and that key panicked; stone C recorded it here
//! and left it, since the predicate is shared with `HashMap`/`HashSet`. Pinned by
//! `tests/diagnostics/probe_ex003_hashability_looks_inside.rs`.
//!
//! **History.** All three verbs used to `panic!`. `new` converted 2026-09-22
//! (excursus 003 stone A, curing the-little-wat F-084: a wat program got a Rust
//! backtrace note, the internal `:rust::` name, and no span). `put`/`get`
//! converted 2026-09-23 (stone C) under the builder's second ruling, *"the least
//! amount of panics possible"* — which SUPERSEDED the arc-109 note's "LEAVE
//! `put`/`get`": its defence, that the checker rejects an opaque-typed key at
//! most call sites, measured false both direct and through a generic `K`. The
//! line is not total-vs-partial and not whose-fault-is-the-input:
//! `:wat::i64::/` is `@Totality Partial` and still refuses INSIDE the language.
//! Record: `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`
//! and `docs/excursus/2026/09/003-the-little-wat-findings/`.
//!
//! The mechanism is `src/rust_deps/sqlite.rs`'s "Errors-as-values — the exact
//! mechanism": `#[wat_dispatch]` marshals `Result<T, E>` natively via the
//! blanket `ToWat`/`FromWat` impls, INCLUDING `Result<Self, E>` for a
//! constructor, through `emit_return_marshal`'s `result_ok_is_self` arm. Zero
//! macro changes were needed for any of the three.

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
    /// A non-hashable key (an opaque handle) is an `Err` `(code, diagnostic, message)` tuple,
    /// never a panic — the same `RawFault` shape `new` returns, with `diagnostic` naming the
    /// **user-facing** verb `:wat::cache::Lru/put` (see `new`'s ⭐ note, and the module doc's
    /// failure surface). Nothing is inserted: the refusal leaves the cache untouched.
    pub fn put(
        &mut self,
        k: Value,
        v: Value,
    ) -> Result<Option<(Value, Value)>, (i64, String, String)> {
        if !value_is_hashable(&k) {
            return Err((
                0,
                ":wat::cache::Lru/put".to_string(),
                format!("key must be a hashable value; got {}", k.type_name()),
            ));
        }
        Ok(self.inner.push(k, v))
    }

    /// `:rust::cache::Lru/get cache k` — `Some(v)` on a hit (which bumps `k`
    /// to MRU), `None` on a miss.
    ///
    /// A non-hashable key (an opaque handle) is a MISS, never a panic: a key `put` refuses to
    /// store cannot be present, so the answer is total and needs no error channel — `get`'s
    /// return type is already `Option`. Precedent: `HashMap`'s `contains-key?` answers `false`
    /// for an unhashable key, "never inserted" (`src/collection/eval.rs`,
    /// `hashmap_contains_key_q_inner`). The guard must stay: `impl Hash for Value` is
    /// `unreachable!()` for these variants, so looking the key up would panic in the hasher.
    pub fn get(&mut self, k: Value) -> Option<Value> {
        if !value_is_hashable(&k) {
            return None;
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

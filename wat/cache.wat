;; wat/cache.wat — Arc 278 Cache Stone 1: the `:wat::cache::Lru` BOUNDED CACHE primitive, a thin
;; honest wat surface over the fresh `:rust::cache::Lru` shim (src/rust_deps/cache.rs).
;;
;; A cache is table-stakes substrate, so it lives in CORE (the same call sqlite and telemetry got).
;; Study oracle: `crates/wat-lru`'s `:wat::lru::LocalCache` — the distributions-of-wat experiment.
;; This is NOT a copy of it (DESIGN-cache-tooling-to-core.md): the names are the ruled
;; `:wat::cache::` ones, and `put` hands back a NAMED record instead of a positional pair.
;;
;; ─── single-thread-owned; zero Mutex ─────────────────────────────────────────────────────────
;; No pipe, no thread, no queue — the fastest memoization possible. The handle is scope-guarded by
;; a thread-id check (`scope = "thread_owned"` on the shim's `#[wat_dispatch]`), which is
;; structural, not contended. The MULTI-CLIENT form is a later stone's `defservice`
;; (`:wat::cache::lru-svc`), where the actor's serialization is the mutex — not a lock here.
;;
;; ─── the named surface (do NOT rename or add verbs) ──────────────────────────────────────────
;; `Lru<K,V>` (opaque, thread-owned) · `Entry<K,V>` (a displaced key/value pair) ·
;; `Lru::new` / `Lru::put` / `Lru::get` / `Lru::len`. The verbs are TYPE-SCOPED under `Lru::` on
;; purpose: the BARE `:wat::cache::get` / `:wat::cache::put` names are reserved for the later
;; stones' client verbs over the cache flavours (`Lru | HolographicLru`), which — per the sqlite
;; `select` precedent — must be ONE `defclause`, since wat forbids two `defn`s sharing an FQDN.
;;
;; ─── `Entry`, not a bare tuple ───────────────────────────────────────────────────────────────
;; The Rust shim returns the displaced pair as `Option<(Value,Value)>` (proven marshaling; tuples
;; have blanket ToWat/FromWat). The oracle stopped there and typed its surface
;; `Option<(K,V)>` — positional, so a caller reads `first`/`second` and has to remember which is
;; which. Modern wat prefers a NAMED aggregate: `put` below lifts that tuple into
;; `:wat::cache::Entry` (`key`/`value`), so the eviction is self-describing at every call site.
;; The lift is the ONE place the positional form is touched.
;;
;; ─── failure surface ─────────────────────────────────────────────────────────────────────────
;; Unlike `:wat::sqlite::*` (every verb errors-as-values), the two guards on this primitive
;; PANIC: a non-positive `capacity` and a non-hashable (opaque-handle) key. That is deliberate
;; behaviour-parity with the oracle for this stone — see src/rust_deps/cache.rs's module doc.
;;
;; Loads after wat/core.wat (typealias/defrecord/defn + Option are core builtins) and after
;; wat/Record.wat is NOT required — `defrecord` is a defmacro, registered in the order-free
;; pre-expansion pass. So this file sits immediately after wat/sqlite.wat, beside the other
;; `:rust::`-shim surface. A baked core source may define under `:wat::` (stdlib bypasses the
;; reserved-prefix gate).

(:wat::core::use! :rust::cache::Lru)

;; ─── the opaque handle — the wat-native name over the :rust:: opaque type ────────────────────
;; unify's alias expansion walks through at every use site, so `:wat::cache::Lru<K,V>` and the
;; backing `:rust::cache::Lru<K,V>` are interchangeable.
(:wat::core::typealias :wat::cache::Lru<K,V> :rust::cache::Lru<K,V>)

;; ─── Entry — a displaced key/value pair, named ───────────────────────────────────────────────
(:wat::core::defrecord :wat::cache::Entry<K,V>
  [key   <- :K
   value <- :V])

;; ─── new ─────────────────────────────────────────────────────────────────────────────────────
;; `capacity` is the hard bound on entry count; it must be positive.
(:wat::core::defn :wat::cache::Lru::new<K,V>
  [capacity <- :wat::core::i64]
  -> :wat::cache::Lru<K,V>
  (:rust::cache::Lru::new capacity))

;; ─── put ─────────────────────────────────────────────────────────────────────────────────────
;; Insert or update, bumping `k` to MRU. Returns the DISPLACED entry — the least-recently-used
;; one when the insert pushed past capacity, or `k`'s previous binding when `k` was already
;; present — and `:wat::core::None` when nothing was displaced.
(:wat::core::defn :wat::cache::Lru::put<K,V>
  [cache <- :wat::cache::Lru<K,V>
   k     <- :K
   v     <- :V]
  -> :wat::core::Option<wat::cache::Entry<K,V>>
  (:wat::core::match (:rust::cache::Lru::put cache k v)
    ((:wat::core::Some pair)
      (:wat::core::Some
        (:wat::cache::Entry :key (:wat::core::first pair) :value (:wat::core::second pair))))
    (:wat::core::None :wat::core::None)))

;; ─── get ─────────────────────────────────────────────────────────────────────────────────────
;; `Some v` on a hit (which bumps `k` to MRU), `None` on a miss.
(:wat::core::defn :wat::cache::Lru::get<K,V>
  [cache <- :wat::cache::Lru<K,V>
   k     <- :K]
  -> :wat::core::Option<V>
  (:rust::cache::Lru::get cache k))

;; ─── len ─────────────────────────────────────────────────────────────────────────────────────
;; Current entry count (never above capacity). Read-only — does not touch LRU order.
(:wat::core::defn :wat::cache::Lru::len<K,V>
  [cache <- :wat::cache::Lru<K,V>]
  -> :wat::core::i64
  (:rust::cache::Lru::len cache))

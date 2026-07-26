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

;; ═══ Stone 2 — :wat::cache::Cache<K,V>, the MULTI-CLIENT `defservice` form ═══════════════════
;;
;; Stone 1 above is thread-owned, zero-mutex — the fastest single-owner memoization. Stone 2 is
;; the SHARED-cache form: a `defservice` whose actor serialization IS the mutex, so N clients
;; share ONE cache with no lock written anywhere — the arc-130 N-client case landing natively.
;; The client verbs the macro generates are `lru-svc/get` / `lru-svc/put` (service-scoped, NOT
;; the bare `:wat::cache::get`/`put` — those names stay RESERVED, per the header above, for a
;; later stone's `defclause` over both cache flavours).
;;
;; ─── the one contract decision — durable holds the SPEC, ephemeral holds the HANDLE ─────────
;; `:wat::cache::Lru` is `scope = "thread_owned"` on the Rust shim (header above): it CANNOT
;; cross a wire or a hibernation boundary, and 293.W enforces exactly that — an impure
;; surface-typed field may live only in `:ephemeral`. So:
;;   `:durable   [capacity <- i64]`        — plain EDN, the SPEC the resource is rebuilt from.
;;   `:ephemeral [cache <- Lru<K,V>]`      — the live handle, born inside `:init` by calling
;;                                           `Lru::new` on the durable capacity.
;; This is R5 at the service layer — store the thunk, not the answer — and it is correct BY
;; CONSTRUCTION, not by discipline: writing the handle into `:durable` instead does not compile
;; (293.W rejects an impure `Lru<K,V>` field outside `:ephemeral`).
(:wat::core::defsurface :wat::cache::Cache<K,V> :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::cache::Cache::GetRequest<K> [key <- :K])
   (:wat::core::defenum :wat::cache::Cache::GetResponse<V> :wat::enum::Pure
     :Hit              [value <- :V]
     :Miss             []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :wat::cache::Cache::PutRequest<K,V> [key <- :K  value <- :V])
   (:wat::core::defenum :wat::cache::Cache::PutResponse<K,V> :wat::enum::Pure
     ;; `displaced` reuses Stone 1's `:wat::cache::Entry<K,V>` — a `:wat::`-prefixed type, so the
     ;; S4c `:messages`-completeness wall treats it as stdlib and does not require it re-declared
     ;; here (it is not a message of THIS surface — it is the shared cache-primitive vocabulary).
     :Ok               [displaced <- :wat::core::Option<wat::cache::Entry<K,V>>]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :wat::cache::Cache<K,V>  req <- :wat::cache::Cache::GetRequest<K>]
     -> :wat::cache::Cache::GetResponse<V> :max-request-bytes 1024)
   (put [self <- :wat::cache::Cache<K,V>  req <- :wat::cache::Cache::PutRequest<K,V>]
     -> :wat::cache::Cache::PutResponse<K,V> :max-request-bytes 1024)])

(:wat::service::defservice :wat::cache::lru-svc<K,V>
  :satisfies :wat::cache::Cache<K,V>
  :durable   [capacity <- :wat::core::i64]
  :ephemeral [cache <- :wat::cache::Lru<K,V>]
  :init (:wat::core::fn [record <- :wat::cache::lru-svc::Record<K,V>]
          -> :wat::cache::lru-svc::State<K,V>
          (:wat::cache::lru-svc::State
            :durable record
            :cache (:wat::cache::Lru::new (:wat::cache::lru-svc::Record/capacity record))))
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s
       (:wat::core::match (:wat::cache::Lru::get (:wat::cache::lru-svc::State/cache s)
                             (:wat::cache::Cache::GetRequest/key req))
         ((:wat::core::Some v) (:wat::cache::Cache::GetResponse::Hit v))
         (:wat::core::None (:wat::cache::Cache::GetResponse::Miss)))))
   ;; `s` is UNCHANGED on Reply — the mutation is inside the opaque `Lru` handle via
   ;; `Lru::put`, not in State (mirrors `wat/query/sqlite-store.wat`'s `conn` pattern).
   (put [s req]
     (:wat::service::Outcome::Reply s
       (:wat::cache::Cache::PutResponse::Ok
         (:wat::cache::Lru::put (:wat::cache::lru-svc::State/cache s)
           (:wat::cache::Cache::PutRequest/key req)
           (:wat::cache::Cache::PutRequest/value req)))))])

;; ═══ Stone 3 — :wat::cache::HolographicLru, the SIMILARITY-KEYED composite ═══════════════════
;;
;; The other cache flavour: same eviction discipline as Stone 1's `Lru<K,V>`, different
;; key-matching — exact-key becomes *hologram similarity*. Concrete over `HolonAST` (the
;; Hologram store is HolonAST-keyed, not generic), so unlike Stone 1 this type carries no `<K,V>`.
;;
;; Study oracle: `crates/wat-holon-lru/wat/holon/lru/HologramCache.wat` — read for the shape
;; (`put`'s eviction → `Hologram/remove` chain, `get`'s `Hologram/find` → LRU-bump), never copied.
;; Rebuilt here clean on Stone 1's `:wat::cache::Lru<K,V>` primitive (named `Entry`, not the
;; oracle's positional tuple) exactly as Stone 1 was rebuilt from its own oracle.
;;
;; ─── the two structures, and the invariant that binds them ──────────────────────────────────
;; `hologram` is the similarity index and holds the VALUES. `lru` is the recency/bound index and
;; holds KEYS ONLY — its value slot is `:wat::core::nil` on purpose: the LRU is not storing
;; anything, it is the bound and the recency order; the values live in the Hologram. The LRU
;; exists so the Hologram cannot grow without limit.
;;
;; THE LOAD-BEARING INVARIANT — dual eviction. The two structures must never disagree about which
;; keys exist. When `put` overflows the LRU, the LRU hands back the displaced `Entry` and that
;; key is removed from the Hologram too (`HolographicLru::put` below). Miss this and the Hologram
;; grows forever while the LRU believes it is bounded — the cache silently stops being a cache.
;;
;; `defstruct`, not `defrecord`: both fields are live impure handles (a `Hologram` and a
;; thread-owned `Lru`). That also means a `HolographicLru` can only ever live in a service's
;; `:ephemeral` — Stone 4's `hologram-svc` problem, not this one's.
;;
;; ─── the verbs (type-scoped, mirroring Stone 1) ──────────────────────────────────────────────
;; `HolographicLru::new` / `::put` / `::get` / `::len`. The BARE `:wat::cache::get` /
;; `:wat::cache::put` names stay RESERVED — a later stone makes them ONE `defclause` over both
;; flavours (`Lru | HolographicLru`), and wat forbids two `defn`s sharing an FQDN. Taking them
;; here would block that stone.

(:wat::core::defstruct :wat::cache::HolographicLru
  [hologram <- :wat::holon::Hologram
   lru      <- :wat::cache::Lru<wat::holon::HolonAST,wat::core::nil>])

;; ─── new ─────────────────────────────────────────────────────────────────────────────────────
;; `filter` gates `Hologram/find` hits (bind `:wat::holon::filter-coincident` /
;; `filter-present` / `filter-accept-any`, or a caller-supplied closure). `capacity` is the LRU's
;; hard bound on entry count — the same guard Stone 1's `Lru::new` carries (must be positive).
(:wat::core::defn :wat::cache::HolographicLru::new
  [filter   <- :wat::core::Fn(wat::core::f64)->wat::core::bool
   capacity <- :wat::core::i64]
  -> :wat::cache::HolographicLru
  (:wat::cache::HolographicLru
    :hologram (:wat::holon::Hologram/make filter)
    :lru (:wat::cache::Lru::new capacity)))

;; ─── put — insert into the Hologram + bump/bound via the LRU, dual-evicting on overflow ───────
;; 1. Insert (key, val) into the Hologram (slot routing is internal).
;; 2. Push key -> nil onto the LRU (V is unit; the LRU only tracks freshness by key).
;; 3. If step 2 displaced an entry (over capacity), remove ITS key from the Hologram too — the
;;    dual-eviction invariant. Without this the Hologram keeps growing after the LRU claims it
;;    dropped something.
(:wat::core::defn :wat::cache::HolographicLru::put
  [store <- :wat::cache::HolographicLru
   key   <- :wat::holon::HolonAST
   val   <- :wat::holon::HolonAST]
  -> :wat::core::nil
  (:wat::core::let
    [hologram (:wat::cache::HolographicLru/hologram store)
     lru (:wat::cache::HolographicLru/lru store)
     _ (:wat::holon::Hologram/put hologram key val)
     evicted (:wat::cache::Lru::put lru key nil)]
    (:wat::core::match evicted
      ((:wat::core::Some entry)
        (:wat::core::let
          [_ (:wat::holon::Hologram/remove hologram (:wat::cache::Entry/key entry))]
          nil))
      (:wat::core::None nil))))

;; ─── get — similarity lookup + LRU bump on hit ─────────────────────────────────────────────────
;; `Hologram/find'` (prime — arc 278 retirement in progress; bare `Hologram/find` is the dying
;; non-prime kept alive only for `crates/wat-holon-lru`'s oracle caller, NEVER reach for it here)
;; returns a `:wat::holon::Match` carrying the MATCHED key (not necessarily `probe` itself — this
;; is what makes the lookup similarity-keyed rather than exact) together with the value. Bump the
;; matched key in the LRU (`Lru::put` on an already-present key updates its recency without
;; displacing anything) and return `Some val`. `None` on a miss (filter rejected, or nothing
;; coincident).
(:wat::core::defn :wat::cache::HolographicLru::get
  [store <- :wat::cache::HolographicLru
   probe <- :wat::holon::HolonAST]
  -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
    [hologram (:wat::cache::HolographicLru/hologram store)
     lru (:wat::cache::HolographicLru/lru store)]
    (:wat::core::match (:wat::holon::Hologram/find' hologram probe)
      ((:wat::core::Some m)
        (:wat::core::let
          [matched-key (:wat::holon::Match/key m)
           val (:wat::holon::Match/value m)
           _ (:wat::cache::Lru::put lru matched-key nil)]
          (:wat::core::Some val)))
      (:wat::core::None :wat::core::None))))

;; ─── len — total entries, read via the Hologram (the value-holding half) ──────────────────────
(:wat::core::defn :wat::cache::HolographicLru::len
  [store <- :wat::cache::HolographicLru]
  -> :wat::core::i64
  (:wat::holon::Hologram/len (:wat::cache::HolographicLru/hologram store)))

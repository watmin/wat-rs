;; wat/cache.wat — Arc 278 Cache Stone 1: the `:wat::cache::Lru` BOUNDED CACHE primitive, a thin
;; honest wat surface over the fresh `:rust::cache::Lru` shim (src/rust_deps/cache.rs).
;;
;; A cache is table-stakes substrate, so it lives in CORE (the same call sqlite and telemetry got).
;; Study oracle (⚠ GONE — Stone 5 annihilated the crate; kept as provenance, not a live path):
;; `crates/wat-lru`'s `:wat::lru::LocalCache` — the distributions-of-wat experiment.
;; NOTE: a STUDY oracle (a prior impl read for shape), NOT rete's `$oracle` differential.
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
;; `(Lru :- [K V])` (opaque, thread-owned) · `(Entry :- [K V])` (a displaced key/value pair) ·
;; `Lru::new` / `Lru::put` / `Lru::get` / `Lru::len`. The verbs are TYPE-SCOPED under `Lru::` on
;; purpose: the BARE `:wat::cache::get` / `:wat::cache::put` names are reserved for the later
;; stones' client verbs over the cache flavours (`Lru | HolographicLru`), which — per the sqlite
;; `select` precedent — must be ONE `defclause`, since wat forbids two `defn`s sharing an FQDN.
;;
;; ─── `Entry`, not a bare tuple ───────────────────────────────────────────────────────────────
;; The Rust shim returns the displaced pair as `Option<(Value,Value)>` (proven marshaling; tuples
;; have blanket ToWat/FromWat). The oracle stopped there and typed its surface
;; `(Option :- [(:wat::core::Tuple :- [K V])])` — positional, so a caller reads `first`/`second` and has to remember which is
;; which. Modern wat prefers a NAMED aggregate: `put` below lifts that tuple into
;; `:wat::cache::Entry` (`key`/`value`), so the eviction is self-describing at every call site.
;; The lift is the ONE place the positional form is touched.
;;
;; ─── failure surface ─────────────────────────────────────────────────────────────────────────
;; Unlike `:wat::sqlite::*` (every verb errors-as-values), the two guards on this primitive
;; PANIC: a non-positive `capacity` and a non-hashable (opaque-handle) key. That is deliberate
;; behaviour-parity with the STUDY oracle for this stone (see `src/rust_deps/cache.rs`'s module
;; doc — and note the crate itself is gone, annihilated by Stone 5). These are the two
;; programming-error inputs, and the checker already rejects an opaque-typed key at most call
;; sites. Whether the no-hidden-failures law should reach them is an OPEN decision, tracked as
;; a row in docs/arc/2026/06/278-rules-engine/NEXT-STRIKES-theater-hunt.md ("exigere — the cache
;; panic conversion"); the reason Stone 1 gave for deferring it has since expired. The full
;; account is src/rust_deps/cache.rs's module doc. Converting moves BOTH files together.
;;
;; Loads after wat/core.wat (typealias/defrecord/defn + Option are core builtins) and after
;; wat/Record.wat is NOT required — `defrecord` is a defmacro, registered in the order-free
;; pre-expansion pass. So this file sits immediately after wat/sqlite.wat, beside the other
;; `:rust::`-shim surface. A baked core source may define under `:wat::` (stdlib bypasses the
;; reserved-prefix gate).
;;
;; ─── BRIEF-cache-batch-surface (arc 278) — BATCH, both directions, TWO deliberate departures ──
;; `docs/CONVENTIONS.md:658` (arc 119) rules that every wat-rs-shipped service's `get`/`put` is
;; BATCH-oriented (Console excepted); `(Cache :- [K V])` (Stone 2/4, below) originally shipped
;; single-key — a miss in the brief that designed it. `(Cache :- [K V])`'s `get`/`put` now take/return
;; `Vector`s. The shapes are SETTLED (builder-ruled 2026-07-26) and deliberately depart from arc
;; 119's OWN prose in two places — this note is why nobody should "correct" them back:
;;   1. Arc 119 says `get -> Vec<Option<V>>`. This ships a NAMED `(Cache::GetResult :- [V])` enum
;;      (`:Hit [value <- V] | :Miss []`) instead — per the later named-enum doctrine ("a proper
;;      enum name is doubly useful"; `Option` tells a reader nothing about the DOMAIN). It is also
;;      the extensible choice: a SIMILARITY cache's miss has kinds (below-threshold vs nothing
;;      stored) that `Option` forecloses ever distinguishing.
;;   2. Arc 119 says `put -> :wat::core::nil`. A bare `nil` is no longer expressible for a
;;      serviceable op — every op-Response must carry `RequestTooLarge`/`RequestMalformed`
;;      (the arc 278 request-shape wall, wat/service.wat). `PutResponse::Ok []` is what `nil`
;;      became; precedent is `:wat::kernel::StdOut::WriteResponse`. This also resolves an honesty
;;      problem: `Lru::put` returns the displaced `Entry` but `HolographicLru::put` (Stone 3/4,
;;      below) returns bare `nil` and never exposes the evicted key — so a `displaced` field on
;;      `PutResponse` could not be told truthfully by both satisfiers. Eviction reporting, if ever
;;      wanted, belongs where it can be honest — not in `put`'s reply; it stays observable through
;;      a later `get` miss (both services' gates prove this).
;; Full reasoning: docs/arc/2026/06/278-rules-engine/BRIEF-cache-batch-surface.md.

(:wat::core::use! :rust::cache::Lru)

;; ─── the opaque handle — the wat-native name over the :rust:: opaque type ────────────────────
;; unify's alias expansion walks through at every use site, so `(:wat::cache::Lru :- [K V])` and the
;; backing `(:rust::cache::Lru :- [K V])` are interchangeable.
(:wat::core::typealias :wat::cache::Lru :- [K V] (:rust::cache::Lru :- [K V]))

;; ─── Entry — a displaced key/value pair, named ───────────────────────────────────────────────
(:wat::core::defrecord :wat::cache::Entry :- [K V]
  [key   <- :K
   value <- :V])

;; ─── new ─────────────────────────────────────────────────────────────────────────────────────
;; `capacity` is the hard bound on entry count; it must be positive.
(:wat::core::defn :wat::cache::Lru::new :- [K V]
  [capacity <- :wat::core::i64]
  -> (:wat::cache::Lru :- [K V])
  (:rust::cache::Lru::new capacity))

;; ─── put ─────────────────────────────────────────────────────────────────────────────────────
;; Insert or update, bumping `k` to MRU. Returns the DISPLACED entry — the least-recently-used
;; one when the insert pushed past capacity, or `k`'s previous binding when `k` was already
;; present — and `:wat::core::None` when nothing was displaced.
(:wat::core::defn :wat::cache::Lru::put :- [K V]
  [cache <- (:wat::cache::Lru :- [K V])
   k     <- :K
   v     <- :V]
  -> (:wat::core::Option :- [(:wat::cache::Entry :- [K V])])
  (:wat::core::match (:rust::cache::Lru::put cache k v)
    ((:wat::core::Some pair)
      (:wat::core::Some
        (:wat::cache::Entry :key (:wat::core::first pair) :value (:wat::core::second pair))))
    (:wat::core::None :wat::core::None)))

;; ─── get ─────────────────────────────────────────────────────────────────────────────────────
;; `Some v` on a hit (which bumps `k` to MRU), `None` on a miss.
(:wat::core::defn :wat::cache::Lru::get :- [K V]
  [cache <- (:wat::cache::Lru :- [K V])
   k     <- :K]
  -> (:wat::core::Option :- [V])
  (:rust::cache::Lru::get cache k))

;; ─── len ─────────────────────────────────────────────────────────────────────────────────────
;; Current entry count (never above capacity). Read-only — does not touch LRU order.
(:wat::core::defn :wat::cache::Lru::len :- [K V]
  [cache <- (:wat::cache::Lru :- [K V])]
  -> :wat::core::i64
  (:rust::cache::Lru::len cache))

;; ═══ Stone 2 — :wat::cache::Cache :- [K V], the MULTI-CLIENT `defservice` form ══════════════
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
;; cross a wire or a hibernation boundary. So:
;;   `:durable   [capacity <- i64]`        — plain EDN, the SPEC the resource is rebuilt from.
;;   `:ephemeral [cache <- (Lru :- [K V])]`      — the live handle, born inside `:init` by calling
;;                                           `Lru::new` on the durable capacity.
;; This is R5 at the service layer — store the thunk, not the answer.
;;
;; ⚠ HISTORY, kept because the claim was wrong for a month and the correction is the point.
;; This paragraph asserted the shape is "correct BY CONSTRUCTION, not by discipline: writing the
;; handle into `:durable` does not compile (293.W rejects an impure `Lru<K,V>` field outside
;; `:ephemeral`)." That was FALSE when written, about the exact type it named: 293.W's containment
;; wall was real and DID reach `:durable`, but `is_pure_type` knew Rust opaques only through
;; hand-written lists, and a PARAMETRIC one fell through `_ => args.iter().all(is_pure_type)` —
;; the CONTAINER presumed pure, only its type args checked. `:durable [c <- Lru<String,i64>]`
;; compiled clean; `Lru<IOWriter,i64>` was correctly refused, which is what proved the container
;; was the miss. Found 2026-08-08 answering the connection-scoped-world stone's STOP-3.
;;
;; ✅ THE CLAIM IS NOW TRUE, and it is enforced rather than asserted. `is_pure_type` consults the
;; `#[wat_dispatch]` opaque registry (`RustDepsRegistry.types`, which every opaque self-registers
;; into) on BOTH arms: a registered Rust opaque is impure, regardless of type arguments. Writing
;; this handle into `:durable` is a load-time `ImpureFieldInPureAggregate`. Standing gate, both
;; directions: `tests/types/probe_arc278_opaque_purity_wall.{rs,wat.bad}` + its `_control.wat`.
;;
;; ⚠ STILL NOT COVERED — do not over-read the above: the wall knows *registered Rust opaques*.
;; `is_pure_type`'s `TypeExpr::Path` arm still ends `None => true`, load-bearing for formal type
;; parameters and for six of our own unregistered-but-pure core types. That is arc 255's registry
;; work. See `293/NOTE-containment-wall-blind-to-rust-opaques.md`.
;; ─── batch, both directions (BRIEF-cache-batch-surface, file header above) ───────────────────
;; `get`: probes IN as a `(Vector :- [K])`, results OUT as an INDEX-ALIGNED `(Vector :- [(GetResult :- [V])])` —
;; `results[i]` answers `probes[i]`. `put`: entries IN as a `(Vector :- [(Entry :- [K V])])`, nothing
;; meaningful out (`:Ok []`) — see the file-header departure note for why. Verbs stay `get`/`put`;
;; the `Vector` in the signature already says batch (no `get-many`/`put-many` split).
;;
;; `:max-request-bytes` is sized for the WORST-case instantiation of this generic surface — the
;; HolonAST-keyed `hologram-svc` (Stone 4, below), whose keys/values are far larger than
;; `lru-svc`'s `String`/`i64`. Measured via `(string::length (edn::write req))` — the SAME
;; expression `wat/service.wat`'s generated guard evaluates — against a live build
;; (`wat-scripts/scratch-pad/probe-arc278-cache-batch-request-bytes.wat`, a throwaway probe run
;; once and deleted after measuring, per the scratch-`.wat` convention). Real numbers:
;;   lru-get (5 String probes)              =   80 bytes
;;   lru-put (5 String/i64 entries)         =  246 bytes
;;   hologram-get (5 HolonAST probes)       =  277 bytes
;;   hologram-put (3 HolonAST/HolonAST entries) =  415 bytes
;;   hologram-put (10 HolonAST/HolonAST entries) = 1222 bytes
;; `2048` on both ops fits a realistic ~16-entry HolonAST-keyed put batch (the worst case) with
;; room to spare, well above what either gate actually sends, and well above the old single-key
;; `1024` cap, which a multi-item HolonAST batch trips immediately by construction.
(:wat::core::defsurface :wat::cache::Cache :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::cache::Cache::GetRequest :- [K] [probes <- (:wat::core::Vector :- [K])])
   (:wat::core::defenum :wat::cache::Cache::GetResult :- [V] :wat::enum::Pure
     :Hit  [value <- :V]
     :Miss [])
   (:wat::core::defenum :wat::cache::Cache::GetResponse :- [V] :wat::enum::Pure
     :Ok               [results <- (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [V])])]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   ;; `(Entry :- [K V])` reuses Stone 1's record — a `:wat::`-prefixed type defined earlier in THIS file
   ;; (before this defsurface), so the S4c `:messages`-completeness wall does not require it
   ;; re-declared here (it is not a message minted BY this surface — it is the shared
   ;; cache-primitive vocabulary, same standing the old single-key `PutResponse`'s `displaced`
   ;; field gave it).
   (:wat::core::defrecord :wat::cache::Cache::PutRequest :- [K V] [entries <- (:wat::core::Vector :- [(:wat::cache::Entry :- [K V])])])
   (:wat::core::defenum :wat::cache::Cache::PutResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- (:wat::cache::Cache :- [K V])  req <- (:wat::cache::Cache::GetRequest :- [K])]
     -> (:wat::cache::Cache::GetResponse :- [V]) :max-request-bytes 2048)
   (put [self <- (:wat::cache::Cache :- [K V])  req <- (:wat::cache::Cache::PutRequest :- [K V])]
     -> :wat::cache::Cache::PutResponse :max-request-bytes 2048)])

(:wat::service::defservice :wat::cache::lru-svc :- [K V]
  :satisfies (:wat::cache::Cache :- [K V])
  :durable   [capacity <- :wat::core::i64]
  :ephemeral [cache <- (:wat::cache::Lru :- [K V])]
  :init (:wat::core::fn [record <- (:wat::cache::lru-svc::Record :- [K V])]
          -> (:wat::cache::lru-svc::State :- [K V])
          (:wat::cache::lru-svc::State
            :durable record
            :cache (:wat::cache::Lru::new (:wat::cache::lru-svc::Record/capacity record))))
  :impls
  ;; Both ops FOLD over the request Vector — `s` is UNCHANGED on Reply either way; the mutation is
  ;; inside the opaque `Lru` handle via `Lru::get`/`Lru::put`, not in State (mirrors
  ;; `wat/query/sqlite-store.wat`'s `conn` pattern). `get`'s fold ACCUMULATES the index-aligned
  ;; results Vector via `conj` — the fold walks `probes` LEFT TO RIGHT and `conj` appends, so
  ;; `results[i]` answers `probes[i]` by construction. `put`'s fold is side-effect-only (dummy
  ;; `nil` accumulator, mirrors `wat/bracket.wat`'s per-item fan-out folds) — `PutResponse` carries
  ;; nothing back (file-header departure note).
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::cache::Cache::GetResponse::Ok
         (:wat::core::foldl
           (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [V])])
                            k   <- :K]
             -> (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [V])])
             (:wat::core::conj acc
               (:wat::core::match (:wat::cache::Lru::get (:wat::cache::lru-svc::State/cache s) k)
                 ((:wat::core::Some v) (:wat::cache::Cache::GetResult::Hit v))
                 (:wat::core::None (:wat::cache::Cache::GetResult::Miss)))))
           (:wat::core::Vector (:wat::cache::Cache::GetResult :- [V]))
           (:wat::cache::Cache::GetRequest/probes req)))))
   (put [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::core::let
         [_ (:wat::core::foldl
              (:wat::core::fn [_acc <- :wat::core::nil
                               e    <- (:wat::cache::Entry :- [K V])]
                -> :wat::core::nil
                (:wat::core::let
                  [_ (:wat::cache::Lru::put (:wat::cache::lru-svc::State/cache s)
                       (:wat::cache::Entry/key e) (:wat::cache::Entry/value e))]
                  nil))
              nil
              (:wat::cache::Cache::PutRequest/entries req))]
         (:wat::cache::Cache::PutResponse::Ok))))])

;; ═══ Stone 3 — :wat::cache::HolographicLru, the SIMILARITY-KEYED composite ═══════════════════
;;
;; The other cache flavour: same eviction discipline as Stone 1's `(Lru :- [K V])`, different
;; key-matching — exact-key becomes *hologram similarity*. Concrete over `HolonAST` (the
;; Hologram store is HolonAST-keyed, not generic), so unlike Stone 1 this type carries no `:- [K V]`.
;;
;; Study oracle (⚠ GONE — Stone 5 annihilated the crate; provenance, not a live path):
;; `crates/wat-holon-lru/wat/holon/lru/HologramCache.wat` — read for the shape
;; (`put`'s eviction → `Hologram/remove` chain, `get`'s `Hologram/find` → LRU-bump), never copied.
;; Rebuilt here clean on Stone 1's `(:wat::cache::Lru :- [K V])` primitive (named `Entry`, not the
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
   lru      <- (:wat::cache::Lru :- [:wat::holon::HolonAST :wat::core::nil])])

;; ─── new ─────────────────────────────────────────────────────────────────────────────────────
;; `filter` gates `Hologram/find` hits (bind `:wat::holon::filter-coincident` /
;; `filter-present` / `filter-accept-any`, or a caller-supplied closure). `capacity` is the LRU's
;; hard bound on entry count — the same guard Stone 1's `Lru::new` carries (must be positive).
(:wat::core::defn :wat::cache::HolographicLru::new
  [filter   <- [:wat::core::f64 :-> :wat::core::bool]
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
;; `Hologram/find`
;; returns a `:wat::holon::Match` carrying the MATCHED key (not necessarily `probe` itself — this
;; is what makes the lookup similarity-keyed rather than exact) together with the value. Bump the
;; matched key in the LRU (`Lru::put` on an already-present key updates its recency without
;; displacing anything) and return `Some val`. `None` on a miss (filter rejected, or nothing
;; coincident).
(:wat::core::defn :wat::cache::HolographicLru::get
  [store <- :wat::cache::HolographicLru
   probe <- :wat::holon::HolonAST]
  -> (:wat::core::Option :- [:wat::holon::HolonAST])
  (:wat::core::let
    [hologram (:wat::cache::HolographicLru/hologram store)
     lru (:wat::cache::HolographicLru/lru store)]
    (:wat::core::match (:wat::holon::Hologram/find hologram probe)
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

;; ═══ Stone 4 — :wat::cache::hologram-svc, the SIMILARITY cache as a SERVICE ═══════════════════
;;
;; The last cache-campaign build. `HolographicLru` behind the SAME `(Cache :- [K V])` surface Stone 2's
;; `lru-svc :- [K V]` wears — but `HolographicLru` is CONCRETE over `HolonAST` (the Hologram store is
;; HolonAST-keyed, not generic; header above), so `hologram-svc` itself carries NO `:- [K V]` — it
;; pins BOTH of `(Cache :- [K V])`'s params to `:wat::holon::HolonAST` at the `:satisfies` site. A
;; concrete, non-parametric service satisfying a parametric surface at FIXED type arguments had
;; no precedent in this corpus before this stone (the only prior `:satisfies` with type args is
;; Stone 2's `lru-svc :- [K V] :satisfies (Cache :- [K V])` — parametric satisfying parametric, service
;; binders flowing straight through). Grounded first as a throwaway probe
;; (`wat-scripts/scratch-pad/probe-arc278-concrete-satisfies-parametric.wat`): a concrete service
;; satisfying `(Cache :- [K V])` at fixed FQDN type args `--check`s clean, and a deliberately-sabotaged
;; variant (swapped K/V) correctly produces type errors naming the exact derived
;; `(Cache::Reply :- [wat::core::String wat::core::i64])` instantiation — proof the macro's
;; `proto-base`/`proto-tp` split (wat/service.wat:268-276) handles a concrete FQDN type-arg list
;; correctly, not just bare binders.
;;
;; ─── the filter problem — a live closure cannot be a durable field ───────────────────────────
;; `HolographicLru::new` (above) takes a `filter <- [f64 :-> bool]` — and every factory that
;; produces one (`filter-coincident` / `filter-present` / `filter-accept-any`, wat/holon.wat:65-93)
;; is a CLOSURE that captures ambient state (`:wat::config::dim-count`) at call time. `Admin::Init`
;; is unconditionally Pure (293.W), so an impure `Fn`-typed argument cannot reach `:init` from
;; `:durable` — the same wall Stone 2's header already states for the `Lru` handle itself, one
;; level up: an impure surface-typed value may live only in `:ephemeral`.
;;
;; The shape that works, precedented by the stdio-as-defservice stones (an fd *number* seed → a
;; live handle born inside `:init` via the whitelisted `from-fd`): `:durable` holds a PURE SEED
;; naming WHICH floor, and `:init` maps the seed to the live filter closure before calling
;; `HolographicLru::new`. The three filters are a CLOSED set — a `:wat::enum::Pure` nullary-variant
;; enum is the honest shape for "one of exactly these three," not a bare keyword (untyped, no
;; exhaustiveness at the `:init` match) or a `String` (same weakness, plus a runtime-only failure
;; mode on a typo). Four questions: Obvious — a reader sees the three names, matching
;; wat/holon.wat's own three factories one-for-one. Simple — one flat enum, no nesting. Honest — it
;; cannot represent a fourth, non-existent filter; a keyword or String could silently carry
;; garbage past construction into `:init`'s match (where it would need a catch-all "else" arm that
;; a bare-keyword-or-String encoding is honest about only by hoping nobody typos). Good UX — the
;; caller writes `:filter (:wat::cache::HologramFilterKind::Coincident)` and the compiler rejects
;; anything else; a `:wat::cache::hologram-svc::Record` literal is self-describing at the call site.
(:wat::core::defenum :wat::cache::HologramFilterKind :wat::enum::Pure
  :Coincident []
  :Present    []
  :AcceptAny  [])

(:wat::service::defservice :wat::cache::hologram-svc
  :satisfies (:wat::cache::Cache :- [:wat::holon::HolonAST :wat::holon::HolonAST])
  :durable   [capacity <- :wat::core::i64
              filter   <- :wat::cache::HologramFilterKind]
  :ephemeral [cache <- :wat::cache::HolographicLru]
  :init (:wat::core::fn [record <- :wat::cache::hologram-svc::Record]
          -> :wat::cache::hologram-svc::State
          (:wat::cache::hologram-svc::State
            :durable record
            :cache (:wat::cache::HolographicLru::new
                     (:wat::core::match (:wat::cache::hologram-svc::Record/filter record)
                       ((:wat::cache::HologramFilterKind::Coincident) (:wat::holon::filter-coincident))
                       ((:wat::cache::HologramFilterKind::Present)    (:wat::holon::filter-present))
                       ((:wat::cache::HologramFilterKind::AcceptAny)  (:wat::holon::filter-accept-any)))
                     (:wat::cache::hologram-svc::Record/capacity record))))
  :impls
  ;; Batch folds, same discipline as `lru-svc` above. `HolographicLru::put` returns `nil` (Stone 3
  ;; header above), unlike `Lru::put` — the dual-eviction chain removes the displaced key from the
  ;; Hologram internally but never hands it back — so this was ALREADY an honest `nil` per-entry,
  ;; before batching; `PutResponse::Ok []` (file-header departure note) now says the same thing at
  ;; the whole-batch level instead of a per-entry `Option`. Eviction is still OBSERVABLE through
  ;; the service — just via a later `get` miss, exactly as the gate proves.
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::cache::Cache::GetResponse::Ok
         (:wat::core::foldl
           (:wat::core::fn [acc   <- (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])])
                            probe <- :wat::holon::HolonAST]
             -> (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])])
             (:wat::core::conj acc
               (:wat::core::match (:wat::cache::HolographicLru::get (:wat::cache::hologram-svc::State/cache s) probe)
                 ((:wat::core::Some v) (:wat::cache::Cache::GetResult::Hit v))
                 (:wat::core::None (:wat::cache::Cache::GetResult::Miss)))))
           (:wat::core::Vector (:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST]))
           (:wat::cache::Cache::GetRequest/probes req)))))
   (put [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat::core::let
         [_ (:wat::core::foldl
              (:wat::core::fn [_acc <- :wat::core::nil
                               e    <- (:wat::cache::Entry :- [:wat::holon::HolonAST :wat::holon::HolonAST])]
                -> :wat::core::nil
                (:wat::core::let
                  [_ (:wat::cache::HolographicLru::put (:wat::cache::hologram-svc::State/cache s)
                       (:wat::cache::Entry/key e) (:wat::cache::Entry/value e))]
                  nil))
              nil
              (:wat::cache::Cache::PutRequest/entries req))]
         (:wat::cache::Cache::PutResponse::Ok))))])

;; wat/query/mem.wat — Arc 278 stone S0 (scope addition): `:wat::query::MemStore` — the FIRST
;; real `:wat::query::Store` / `ReadStore` satisfier.
;;
;; Dual-purpose per the strike: a genuine in-memory backend AND the oracle sqlite (`:wat::sqlite'`,
;; a later stone) will be differential-tested against — correct-by-construction, not a canned stub.
;;
;; ── the state model — a defservice actor, NOT a raw mutable cell ─────────────────────────────
;; `put` must mutate durable state visible to a LATER, SEPARATE `scan` call on the same store —
;; genuine interior mutability behind a `:wat::core::Struct`. wat has no generic mutable-cell
;; primitive (no `Cell`/`atom`/`swap!`); the tempting workaround — a `defstruct` holding BOTH ends
;; of one `make-channel` pair as a single-place "MVar" — is EXPLICITLY REJECTED by the compiler's
;; `ChannelPairDeadlock` static check (src/check.rs:2774-2837): any call site holding a `Sender<T>`
;; and a `Receiver<T>` that trace to the same `make-channel` anchor is a hard type error ("Holding
;; both ends of one channel in one role deadlocks any recv"). Confirmed empirically — the same
;; shape rejected at `probe::query::MemStore` construction in a throwaway probe, span pointing at
;; the offending constructor call.
;;
;; The sanctioned answer is `:wat::service::defservice` (wat/service.wat, arc 209/291): a spawned
;; actor holds durable state in its own tail-recursive `serve` loop parameter (rete's own
;; Session-threading convention — `Outcome::Reply new-state reply` rebinds `state` for the next
;; iteration; no mutable cell anywhere), and callers talk to it over a connected `Peer'<Op,Reply>`.
;; `put`/`scan`/`scan-index` become client RPCs; the actor's loop is the ONE place mutation
;; "happens" (by rebinding, not by mutating memory).
;;
;; ── a real substrate finding: `start` cannot be factored into a reusable constructor fn ────────
;; `(:wat::spawn::thread)` ties the spawned service thread's lifetime to the LEXICAL SCOPE of the
;; `start` call — if `start` + `connect'` run inside a separate `MemStore::new`-style helper
;; function and the wrapper is returned to the caller, the connection is already dead by the time
;; the caller uses it (`recv'`/`send'` report "channel disconnected"), even though the `Peer'`
;; value itself is still held (confirmed empirically by isolating the exact reproduction: a bare
;; `Peer'` returned from a helper function fails identically to the wrapped-struct case; the SAME
;; construction inlined in the caller's own `let` succeeds). So: `start` + `connect'` + every call
;; through the resulting peer must share one lexical scope (or an ancestor block that outlives all
;; of them) — see `probes/../deftest'` gate, which inlines construction for exactly this reason.
;; A convenience constructor is future work once/if the substrate grows a scope-detach primitive.
;;
;; ── the wire format ─────────────────────────────────────────────────────────────────────────
;; The service's durable record is one `PersistentVector<StoredRow>` — the whole table, unindexed;
;; `scan`/`scan-index` filter + `sort-by` + `take` a plain materialized copy on every read (no
;; separate sorted structure — correct, not fast; a later stone may add per-index structures).
;; `:wat::spawn::thread` locus only (in-memory; no cross-process EDN encoding of `StoredRow`/HashMap
;; needed). NAME provisional (`MemStore` / `mem-store'`) — orchestrator casts intueri later.

;; ─── small pure helpers — filter/sort predicates shared by scan + scan-index ────────────────
(:wat::core::defn :wat::query::sk-after-cursor?
  [sk <- :wat::core::String cursor <- (:wat::core::Option :wat::core::String)] -> :wat::core::bool
  (:wat::core::match cursor -> :wat::core::bool
    (:wat::core::None true)
    ((:wat::core::Some c) (:wat::core::> sk c))))

(:wat::core::defn :wat::query::row-in-range?
  [row <- :wat::query::StoredRow pk <- :wat::core::String lo <- :wat::core::String
   hi <- :wat::core::String cursor <- (:wat::core::Option :wat::core::String)] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::= (:wat::query::StoredRow/pk row) pk)
    (:wat::core::>= (:wat::query::StoredRow/sk row) lo)
    (:wat::core::<= (:wat::query::StoredRow/sk row) hi)
    (:wat::query::sk-after-cursor? (:wat::query::StoredRow/sk row) cursor)))

(:wat::core::defn :wat::query::StoredRow->Row [r <- :wat::query::StoredRow] -> :wat::query::Row
  (:wat::query::Row (:wat::query::StoredRow/pk r) (:wat::query::StoredRow/sk r) (:wat::query::StoredRow/data r)))

;; row's projected (ipk,isk) for a named index, if it declared one — None if the row never
;; projected into this GSI.
(:wat::core::defn :wat::query::row-index-key
  [row <- :wat::query::StoredRow index <- :wat::core::String] -> (:wat::core::Option :wat::query::IndexKey)
  (:wat::core::HashMap/get (:wat::query::StoredRow/index-keys row) index))

(:wat::core::defn :wat::query::index-key-in-range?
  [ik <- :wat::query::IndexKey ipk <- :wat::core::String lo <- :wat::core::String
   hi <- :wat::core::String cursor <- (:wat::core::Option :wat::core::String)] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::= (:wat::query::IndexKey/ipk ik) ipk)
    (:wat::core::>= (:wat::query::IndexKey/isk ik) lo)
    (:wat::core::<= (:wat::query::IndexKey/isk ik) hi)
    (:wat::query::sk-after-cursor? (:wat::query::IndexKey/isk ik) cursor)))

(:wat::core::defn :wat::query::StoredRow->IndexRow
  [r <- :wat::query::StoredRow ik <- :wat::query::IndexKey] -> :wat::query::IndexRow
  (:wat::query::IndexRow (:wat::query::StoredRow/pk r) (:wat::query::StoredRow/sk r)
    (:wat::query::IndexKey/ipk ik) (:wat::query::IndexKey/isk ik) (:wat::query::StoredRow/data r)))

;; ─── the MemStore SERVICE — the real, mutating in-memory backend ────────────────────────────
;; durable = one flat PersistentVector<StoredRow>; `put` conj's the batch on (rete-style pure
;; threading: the `serve` loop rebinds `state` to the returned new State — see wat/service.wat's
;; tail-recursive dispatch, `Outcome::Reply`); `scan`/`scan-index` are pure reads (state
;; unchanged) that filter+sort+paginate a plain materialized copy.
(:wat::service::defservice :wat::query::mem-store'
  :durable [rows <- :wat::core::PersistentVector<wat::query::StoredRow>]
  :ephemeral []
  :ops
  [(:EnsureSchema [s <- :State table <- :wat::query::TableSchema
                   indexes <- (:wat::core::Vector :wat::query::IndexSchema)]
     -> [ok <- :wat::core::bool]
     ;; idempotent no-op — MemStore has no physical schema to establish (the contract's promise
     ;; is satisfied trivially; sqlite's satisfier is where CREATE TABLE/INDEX actually happens).
     (:wat::service::Outcome::Reply s (:wat::query::mem-store'::EnsureSchemaResponse true)))

   (:Put [s <- :State new-rows <- (:wat::core::Vector :wat::query::StoredRow)]
     -> [ok <- :wat::core::bool]
     (:wat::core::let
       [merged (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :wat::query::StoredRow)
                                  r   <- :wat::query::StoredRow]
                   -> (:wat::core::PersistentVector :wat::query::StoredRow)
                   (:wat::core::PersistentVector/conj acc r))
                 (:wat::query::mem-store'::Record/rows (:wat::query::mem-store'::State/durable s))
                 new-rows)]
       (:wat::service::Outcome::Reply
         (:wat::query::mem-store'::State (:wat::query::mem-store'::Record merged))
         (:wat::query::mem-store'::PutResponse true))))

   (:Scan [s <- :State q <- :wat::query::ScanRequest]
     -> [page <- :wat::query::Page]
     (:wat::core::let
       [pk  (:wat::query::ScanRequest/pk q)
        lo  (:wat::query::ScanRequest/sk-lo q)
        hi  (:wat::query::ScanRequest/sk-hi q)
        lim (:wat::query::ScanRequest/limit q)
        cur (:wat::query::ScanRequest/cursor q)
        matches (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :wat::query::Row) r <- :wat::query::StoredRow]
                    -> (:wat::core::Vector :wat::query::Row)
                    (:wat::core::if (:wat::query::row-in-range? r pk lo hi cur)
                      (:wat::core::conj acc (:wat::query::StoredRow->Row r))
                      acc))
                  (:wat::core::Vector :wat::query::Row)
                  (:wat::query::mem-store'::Record/rows (:wat::query::mem-store'::State/durable s)))
        sorted   (:wat::core::sort-by :wat::query::Row/sk matches)
        limited  (:wat::core::into [] (:wat::core::take sorted lim))
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last limited) "scan: limited non-empty when full")))
                   :wat::core::None)]
       (:wat::service::Outcome::Reply s (:wat::query::mem-store'::ScanResponse (:wat::query::Page limited next-cur)))))

   (:ScanIndex [s <- :State q <- :wat::query::IndexScanRequest]
     -> [page <- :wat::query::IndexPage]
     (:wat::core::let
       [index (:wat::query::IndexScanRequest/index q)
        ipk   (:wat::query::IndexScanRequest/ipk q)
        lo    (:wat::query::IndexScanRequest/isk-lo q)
        hi    (:wat::query::IndexScanRequest/isk-hi q)
        lim   (:wat::query::IndexScanRequest/limit q)
        cur   (:wat::query::IndexScanRequest/cursor q)
        matches (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :wat::query::IndexRow) r <- :wat::query::StoredRow]
                    -> (:wat::core::Vector :wat::query::IndexRow)
                    (:wat::core::match (:wat::query::row-index-key r index) -> :wat::core::Vector<wat::query::IndexRow>
                      (:wat::core::None acc)
                      ((:wat::core::Some ik)
                        (:wat::core::if (:wat::query::index-key-in-range? ik ipk lo hi cur)
                          (:wat::core::conj acc (:wat::query::StoredRow->IndexRow r ik))
                          acc))))
                  (:wat::core::Vector :wat::query::IndexRow)
                  (:wat::query::mem-store'::Record/rows (:wat::query::mem-store'::State/durable s)))
        sorted   (:wat::core::sort-by :wat::query::IndexRow/isk matches)
        limited  (:wat::core::into [] (:wat::core::take sorted lim))
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last limited) "scan-index: limited non-empty when full")))
                   :wat::core::None)]
       (:wat::service::Outcome::Reply s (:wat::query::mem-store'::ScanIndexResponse (:wat::query::IndexPage limited next-cur)))))])

;; ─── the wrapper — extend-types the connected client peer to Store / ReadStore ──────────────
;; A satisfier's `self` must be a value the caller constructs once (with `start`+`connect'`
;; INLINE, per the NOTE above) and threads through every call; MemStore just carries the peer.
(:wat::core::defstruct :wat::query::MemStore
  [peer <- :wat::kernel::Peer'<wat::query::mem-store'::Op,wat::query::mem-store'::Reply>])

(:wat::core::extend-type :wat::query::MemStore :wat::query::Store
  (ensure-schema [self table indexes]
    (:wat::core::let
      [_r (:wat::query::mem-store'/ensure-schema (:wat::query::MemStore/peer self)
            (:wat::query::mem-store'/ensure-schema-request table indexes))]
      (:wat::core::Ok nil)))
  (put [self rows]
    (:wat::core::let
      [_r (:wat::query::mem-store'/put (:wat::query::MemStore/peer self)
            (:wat::query::mem-store'/put-request rows))]
      (:wat::core::Ok nil)))
  (scan [self q]
    (:wat::core::let
      [r (:wat::query::mem-store'/scan (:wat::query::MemStore/peer self)
           (:wat::query::mem-store'/scan-request q))]
      (:wat::core::Ok (:wat::query::mem-store'::ScanResponse/page r))))
  (scan-index [self q]
    (:wat::core::let
      [r (:wat::query::mem-store'/scan-index (:wat::query::MemStore/peer self)
           (:wat::query::mem-store'/scan-index-request q))]
      (:wat::core::Ok (:wat::query::mem-store'::ScanIndexResponse/page r)))))

;; `ReadStore`'s `scan`/`scan-index` share the exact same name + shape as `Store`'s — a second
;; `extend-type :wat::query::MemStore :wat::query::ReadStore` re-declaring them would collide
;; ("duplicate define: :wat::query::MemStore/scan already registered"; confirmed empirically).
;; `derive` is extend-type's edge-only half (registers the subtype/satisfaction edge without a
;; method-impl block) — MemStore already HAS scan/scan-index from the Store impl above; derive
;; just tells the checker MemStore ALSO satisfies ReadStore, and both `Store/scan` and
;; `ReadStore/scan` dispatch to the one `:wat::query::MemStore/scan` definition (confirmed).
(:wat::core::derive :wat::query::MemStore :wat::query::ReadStore)

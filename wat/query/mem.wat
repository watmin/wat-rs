;; wat/query/mem.wat — Arc 278 stone S4: `:wat::query::mem-store'` — the FIRST real
;; `:wat::query::Store` satisfier, migrated to the services-as-surfaces OPERATION MODEL
;; (`:satisfies :wat::query::Store` + `:impls`). A dialed `mem-store'` peer IS the Store
;; INTRINSICALLY (arc 293 Path B) — no `MemStore` wrapper struct, no `extend-type`.
;;
;; Dual-purpose per the strike: a genuine in-memory backend AND the oracle sqlite (`:wat::sqlite'`,
;; a later stone) will be differential-tested against — correct-by-construction, not a canned stub.
;;
;; ── the state model — a defservice actor, NOT a raw mutable cell ─────────────────────────────
;; `put` must mutate durable state visible to a LATER, SEPARATE `scan` call on the same store —
;; genuine interior mutability behind a `:wat::core::Struct`. wat has no generic mutable-cell
;; primitive (no `Cell`/`atom`/`swap!`); the tempting workaround — a `defstruct` holding BOTH ends
;; of one channel pair as a single-place "MVar" — deadlocks: a role holding both a
;; `(Sender :- [T])` and its paired `(Receiver :- [T])` keeps the channel alive against its own `recv`, so the
;; recv never wakes. A `ChannelPairDeadlock` static check once rejected this shape at compile time;
;; that walker was RETIRED once locus became reachable only through `defservice` and brackets —
;; the workaround is no longer detected because it is no longer constructible. The deadlock is
;; still real; the shape simply has no way into the substrate any more.
;;
;; The sanctioned answer is `:wat::service::defservice` (wat/service.wat, arc 209/291): a spawned
;; actor holds durable state in its own tail-recursive `serve` loop parameter (rete's own
;; Session-threading convention — `Outcome::Reply new-state reply` rebinds `state` for the next
;; iteration; no mutable cell anywhere), and callers talk to it over a connected `(Peer' :- [Op Reply])`.
;; `put`/`scan`/`scan-index` become client RPCs; the actor's loop is the ONE place mutation
;; "happens" (by rebinding, not by mutating memory).
;;
;; ── a real substrate finding: `start` cannot be factored into a reusable constructor fn ────────
;; `(:wat::spawn::thread)` ties the spawned service thread's lifetime to the LEXICAL SCOPE of the
;; `start` call — if `start` + `connect'` run inside a separate constructor-style helper function
;; and the peer is returned to the caller, the connection is already dead by the time the caller
;; uses it (`recv'`/`send'` report "channel disconnected"), even though the `Peer'` value itself
;; is still held (confirmed empirically by isolating the exact reproduction: a bare `Peer'`
;; returned from a helper function fails identically to the wrapped-struct case; the SAME
;; construction inlined in the caller's own `let` succeeds). So: `start` + `connect'` + every call
;; through the resulting peer must share one lexical scope (or an ancestor block that outlives all
;; of them) — see `probes/../deftest'` gate, which inlines construction for exactly this reason.
;; A convenience constructor is future work once/if the substrate grows a scope-detach primitive.
;;
;; ── the wire format ─────────────────────────────────────────────────────────────────────────
;; The service's durable record is one `(PersistentVector :- [StoredRow])` — the whole table, unindexed;
;; `scan`/`scan-index` filter + `sort-by` + `take` a plain materialized copy on every read (no
;; separate sorted structure — correct, not fast; a later stone may add per-index structures).
;; `:wat::spawn::thread` locus only (in-memory; no cross-process EDN encoding of `StoredRow`/HashMap
;; needed).

;; ─── small pure helpers — filter/sort predicates shared by scan + scan-index ────────────────
(:wat::core::defn :wat::query::sk-after-cursor?
  [sk <- :wat::core::String cursor <- (:wat::core::Option :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::match cursor 
    (:wat::core::None true)
    ((:wat::core::Some c) (:wat::core::> sk c))))

(:wat::core::defn :wat::query::row-in-range?
  [row <- :wat::query::StoredRow pk <- :wat::core::String lo <- :wat::core::String
   hi <- :wat::core::String cursor <- (:wat::core::Option :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::= (:wat::query::StoredRow/pk row) pk)
    (:wat::core::>= (:wat::query::StoredRow/sk row) lo)
    (:wat::core::<= (:wat::query::StoredRow/sk row) hi)
    (:wat::query::sk-after-cursor? (:wat::query::StoredRow/sk row) cursor)))

(:wat::core::defn :wat::query::StoredRow->Row [r <- :wat::query::StoredRow] -> :wat::query::Row
  (:wat::query::Row :pk (:wat::query::StoredRow/pk r) :sk (:wat::query::StoredRow/sk r) :data (:wat::query::StoredRow/data r)))

;; row's projected (ipk,isk) for a named index, if it declared one — None if the row never
;; projected into this GSI.
(:wat::core::defn :wat::query::row-index-key
  [row <- :wat::query::StoredRow index <- :wat::core::String] -> (:wat::core::Option :- [:wat::query::IndexKey])
  (:wat::hashmap::get (:wat::query::StoredRow/index-keys row) index))

(:wat::core::defn :wat::query::index-key-in-range?
  [ik <- :wat::query::IndexKey ipk <- :wat::core::String lo <- :wat::core::String
   hi <- :wat::core::String cursor <- (:wat::core::Option :- [:wat::core::String])] -> :wat::core::bool
  (:wat::core::and
    (:wat::core::= (:wat::query::IndexKey/ipk ik) ipk)
    (:wat::core::>= (:wat::query::IndexKey/isk ik) lo)
    (:wat::core::<= (:wat::query::IndexKey/isk ik) hi)
    (:wat::query::sk-after-cursor? (:wat::query::IndexKey/isk ik) cursor)))

(:wat::core::defn :wat::query::StoredRow->IndexRow
  [r <- :wat::query::StoredRow ik <- :wat::query::IndexKey] -> :wat::query::IndexRow
  (:wat::query::IndexRow :pk (:wat::query::StoredRow/pk r) :sk (:wat::query::StoredRow/sk r)
    :ipk (:wat::query::IndexKey/ipk ik) :isk (:wat::query::IndexKey/isk ik) :data (:wat::query::StoredRow/data r)))

;; ─── the mem-store' SERVICE — the real, mutating in-memory backend ──────────────────────────
;; durable = one flat (PersistentVector :- [StoredRow]); `put` conj's the batch on (rete-style pure
;; threading: the `serve` loop rebinds `state` to the returned new State — see wat/service.wat's
;; tail-recursive dispatch, `Outcome::Reply`); `scan`/`scan-index` are pure reads (state
;; unchanged) that filter+sort+paginate a plain materialized copy. `:satisfies :wat::query::Store`
;; puts this on the operation model: each impl is `(<op> [s req] body)` — `req` is the
;; `Store::<Op>Request` record; the body returns the `Store::<Op>Response` outcome enum via
;; `Outcome::Reply`. MemStore never errors — always `:Success`.
(:wat::service::defservice :wat::query::mem-store
  :satisfies :wat::query::Store
  ;; arc 278 Stone 1b — the per-service hard frame limit FOO (bytes-per-read): the store backs BULK
  ;; writes (the journal forwards batches here), so it declares 10 MiB. Threaded to accepted-connection
  ;; receivers; a frame over this → a reasoned 400 + close, not mute. (512 KiB default is too small.)
  :max-frame-bytes 10485760
  :durable [rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  :ephemeral []
  :impls
  [(ensure-schema [s ctx req]
     ;; idempotent no-op — mem-store' has no physical schema to establish (the contract's
     ;; promise is satisfied trivially; sqlite's satisfier is where CREATE TABLE/INDEX happens).
     (:wat::service::Outcome::Reply s (:wat::query::Store::EnsureSchemaResponse::Success)))

   (put [s ctx req]
     (:wat::core::let
       [new-rows (:wat::query::Store::PutRequest/rows req)
        merged (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                                  r   <- :wat::query::StoredRow]
                   -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                   (:wat::vector::conj acc r))
                 (:wat::query::mem-store::Record/rows (:wat::query::mem-store::State/durable s))
                 new-rows)]
       (:wat::service::Outcome::Reply
         (:wat::query::mem-store::State (:wat::query::mem-store::Record merged))
         (:wat::query::Store::PutResponse::Success))))

   (scan [s ctx req]
     (:wat::core::let
       [pk  (:wat::query::Store::ScanRequest/pk req)
        lo  (:wat::query::Store::ScanRequest/sk-lo req)
        hi  (:wat::query::Store::ScanRequest/sk-hi req)
        lim (:wat::query::Store::ScanRequest/limit req)
        cur (:wat::query::Store::ScanRequest/cursor req)
        matches (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::Row]) r <- :wat::query::StoredRow]
                    -> (:wat::core::Vector :- [:wat::query::Row])
                    (:wat::core::if (:wat::query::row-in-range? r pk lo hi cur)
                      (:wat::core::conj acc (:wat::query::StoredRow->Row r))
                      acc))
                  (:wat::core::Vector :- [:wat::query::Row])
                  (:wat::query::mem-store::Record/rows (:wat::query::mem-store::State/durable s)))
        sorted   (:wat::core::sort-by :wat::query::Row/sk matches)
        limited  (:wat::core::into [] (:wat::core::take sorted lim))
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last limited) "scan: limited non-empty when full")))
                   :wat::core::None)]
       (:wat::service::Outcome::Reply s (:wat::query::Store::ScanResponse::Success limited next-cur))))

   (scan-index [s ctx req]
     (:wat::core::let
       [index (:wat::query::Store::ScanIndexRequest/index req)
        ipk   (:wat::query::Store::ScanIndexRequest/ipk req)
        lo    (:wat::query::Store::ScanIndexRequest/isk-lo req)
        hi    (:wat::query::Store::ScanIndexRequest/isk-hi req)
        lim   (:wat::query::Store::ScanIndexRequest/limit req)
        cur   (:wat::query::Store::ScanIndexRequest/cursor req)
        matches (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::query::IndexRow]) r <- :wat::query::StoredRow]
                    -> (:wat::core::Vector :- [:wat::query::IndexRow])
                    (:wat::core::match (:wat::query::row-index-key r index) 
                      (:wat::core::None acc)
                      ((:wat::core::Some ik)
                        (:wat::core::if (:wat::query::index-key-in-range? ik ipk lo hi cur)
                          (:wat::core::conj acc (:wat::query::StoredRow->IndexRow r ik))
                          acc))))
                  (:wat::core::Vector :- [:wat::query::IndexRow])
                  (:wat::query::mem-store::Record/rows (:wat::query::mem-store::State/durable s)))
        sorted   (:wat::core::sort-by :wat::query::IndexRow/isk matches)
        limited  (:wat::core::into [] (:wat::core::take sorted lim))
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last limited) "scan-index: limited non-empty when full")))
                   :wat::core::None)]
       (:wat::service::Outcome::Reply s (:wat::query::Store::ScanIndexResponse::Success limited next-cur))))])

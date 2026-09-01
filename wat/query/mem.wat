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
;; Durable is one `(PersistentVector :- [StoredRow])` — the soul, the EDN that crosses a wire
;; and survives hibernation. This service runs at thread *or* process locus (`circuit.wat`
;; starts it at `:wat::spawn::process`). The durable shape does not change.
;;
;; Reads are served from an ephemeral index derived at `:init` and maintained on `put`/`delete`:
;; base rows partitioned by `pk` and ordered by `sk`; GSI rows partitioned per index-name by
;; `ipk` and ordered by `isk`. `scan`/`scan-index` are lookup + range + take — O(result), not
;; O(table). Hibernate/resume rebuilds the index from the table by construction (`:init` is
;; the only builder). `scan-index` is the hot path (queue `receive` is a scan-index on
;; `by-visible-at`).

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

;; a Key names the same (pk, sk) a StoredRow occupies — used by `delete` to drop rows
;; without a read. The ephemeral index is derived from StoredRow, so dropping the row
;; from the durable table and from every partition it occupied drops its GSI projection.
(:wat::core::defn :wat::query::key-hits-row?
  [k <- :wat::query::Key r <- :wat::query::StoredRow] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::query::Key/pk k) (:wat::query::StoredRow/pk r))
    (:wat::core::= (:wat::query::Key/sk k) (:wat::query::StoredRow/sk r))
    false))

(:wat::core::defn :wat::query::row-in-delete-batch?
  [r <- :wat::query::StoredRow keys <- (:wat::core::Vector :- [:wat::query::Key])] -> :wat::core::bool
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::bool k <- :wat::query::Key] -> :wat::core::bool
      (:wat::core::if acc true (:wat::query::key-hits-row? k r)))
    false
    keys))

;; ─── ephemeral index — derived at :init, maintained on put/delete ──────────────────────────
;; by-pk:    pk -> rows ordered by sk
;; by-index: index-name -> ipk -> rows ordered by isk
(:wat::core::defrecord :wat::query::MemIndex
  [by-pk    <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::PersistentVector :- [:wat::query::StoredRow])])
   by-index <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::HashMap :- [:wat::core::String (:wat::core::PersistentVector :- [:wat::query::StoredRow])])])
   live     <- (:wat::core::HashMap :- [:wat::core::String (:wat::core::HashMap :- [:wat::core::String :wat::query::StoredRow])])
   pk-dirty <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])])

(:wat::core::defrecord :wat::query::MemInsert
  [rows   <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
   placed <- :wat::core::bool])

(:wat::core::defn :wat::query::empty-stored-rows []
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::PersistentVector :- [:wat::query::StoredRow]))

(:wat::core::defn :wat::query::empty-mem-index [] -> :wat::query::MemIndex
  (:wat::query::MemIndex
    :by-pk (:wat::core::HashMap :- [:wat::core::String (:wat::core::PersistentVector :- [:wat::query::StoredRow])])
    :by-index (:wat::core::HashMap :- [:wat::core::String (:wat::core::HashMap :- [:wat::core::String (:wat::core::PersistentVector :- [:wat::query::StoredRow])])])
    :live (:wat::core::HashMap :- [:wat::core::String (:wat::core::HashMap :- [:wat::core::String :wat::query::StoredRow])])
    :pk-dirty (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])))

(:wat::core::defn :wat::query::mem-pk-rows
  [idx <- :wat::query::MemIndex pk <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::match (:wat::hashmap::get (:wat::query::MemIndex/by-pk idx) pk)
    (:wat::core::None (:wat::query::empty-stored-rows))
    ((:wat::core::Some v) v)))

(:wat::core::defn :wat::query::mem-gsi-rows
  [idx <- :wat::query::MemIndex name <- :wat::core::String ipk <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::match (:wat::hashmap::get (:wat::query::MemIndex/by-index idx) name)
    (:wat::core::None (:wat::query::empty-stored-rows))
    ((:wat::core::Some inner)
      (:wat::core::match (:wat::hashmap::get inner ipk)
        (:wat::core::None (:wat::query::empty-stored-rows))
        ((:wat::core::Some v) v)))))

(:wat::core::defn :wat::query::mem-set-pk-rows
  [idx <- :wat::query::MemIndex pk <- :wat::core::String
   rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  -> :wat::query::MemIndex
  (:wat::query::MemIndex
    :by-pk (:wat::hashmap::assoc (:wat::query::MemIndex/by-pk idx) pk rows)
    :by-index (:wat::query::MemIndex/by-index idx)
    :live (:wat::query::MemIndex/live idx)
    :pk-dirty (:wat::query::MemIndex/pk-dirty idx)))

(:wat::core::defn :wat::query::mem-set-gsi-rows
  [idx <- :wat::query::MemIndex name <- :wat::core::String ipk <- :wat::core::String
   rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  -> :wat::query::MemIndex
  (:wat::core::let
    [outer (:wat::query::MemIndex/by-index idx)
     inner (:wat::core::match (:wat::hashmap::get outer name)
             (:wat::core::None (:wat::core::HashMap :- [:wat::core::String (:wat::core::PersistentVector :- [:wat::query::StoredRow])]))
             ((:wat::core::Some m) m))]
    (:wat::query::MemIndex
      :by-pk (:wat::query::MemIndex/by-pk idx)
      :by-index (:wat::hashmap::assoc outer name (:wat::hashmap::assoc inner ipk rows))
      :live (:wat::query::MemIndex/live idx)
      :pk-dirty (:wat::query::MemIndex/pk-dirty idx))))

(:wat::core::defn :wat::query::mem-mark-pk-dirty
  [idx <- :wat::query::MemIndex pk <- :wat::core::String] -> :wat::query::MemIndex
  (:wat::query::MemIndex
    :by-pk (:wat::query::MemIndex/by-pk idx)
    :by-index (:wat::query::MemIndex/by-index idx)
    :live (:wat::query::MemIndex/live idx)
    :pk-dirty (:wat::hashmap::assoc (:wat::query::MemIndex/pk-dirty idx) pk true)))

(:wat::core::defn :wat::query::mem-mark-pk-clean
  [idx <- :wat::query::MemIndex pk <- :wat::core::String] -> :wat::query::MemIndex
  (:wat::query::MemIndex
    :by-pk (:wat::query::MemIndex/by-pk idx)
    :by-index (:wat::query::MemIndex/by-index idx)
    :live (:wat::query::MemIndex/live idx)
    :pk-dirty (:wat::hashmap::assoc (:wat::query::MemIndex/pk-dirty idx) pk false)))

(:wat::core::defn :wat::query::live-get
  [idx <- :wat::query::MemIndex pk <- :wat::core::String sk <- :wat::core::String]
  -> (:wat::core::Option :- [:wat::query::StoredRow])
  (:wat::core::match (:wat::hashmap::get (:wat::query::MemIndex/live idx) pk)
    (:wat::core::None :wat::core::None)
    ((:wat::core::Some inner) (:wat::hashmap::get inner sk))))

(:wat::core::defn :wat::query::live-assoc
  [idx <- :wat::query::MemIndex pk <- :wat::core::String sk <- :wat::core::String
   row <- :wat::query::StoredRow]
  -> :wat::query::MemIndex
  (:wat::core::let
    [outer (:wat::query::MemIndex/live idx)
     inner (:wat::core::match (:wat::hashmap::get outer pk)
             (:wat::core::None (:wat::core::HashMap :- [:wat::core::String :wat::query::StoredRow]))
             ((:wat::core::Some m) m))]
    (:wat::query::MemIndex
      :by-pk (:wat::query::MemIndex/by-pk idx)
      :by-index (:wat::query::MemIndex/by-index idx)
      :live (:wat::hashmap::assoc outer pk (:wat::hashmap::assoc inner sk row))
      :pk-dirty (:wat::query::MemIndex/pk-dirty idx))))

(:wat::core::defn :wat::query::live-dissoc
  [idx <- :wat::query::MemIndex pk <- :wat::core::String sk <- :wat::core::String]
  -> :wat::query::MemIndex
  (:wat::core::match (:wat::hashmap::get (:wat::query::MemIndex/live idx) pk)
    (:wat::core::None idx)
    ((:wat::core::Some inner)
      (:wat::query::MemIndex
        :by-pk (:wat::query::MemIndex/by-pk idx)
        :by-index (:wat::query::MemIndex/by-index idx)
        :live (:wat::hashmap::assoc (:wat::query::MemIndex/live idx) pk (:wat::core::dissoc inner sk))
        :pk-dirty (:wat::query::MemIndex/pk-dirty idx)))))

(:wat::core::defn :wat::query::rows-without-key
  [rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow]) k <- :wat::query::Key]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                     r   <- :wat::query::StoredRow]
      -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
      (:wat::core::if (:wat::query::key-hits-row? k r)
        acc
        (:wat::vector::conj acc r)))
    (:wat::query::empty-stored-rows)
    rows))

(:wat::core::defn :wat::query::row-isk
  [index <- :wat::core::String row <- :wat::query::StoredRow] -> :wat::core::String
  (:wat::query::IndexKey/isk
    (:wat::core::Option/expect
      (:wat::query::row-index-key row index)
      "mem-store: indexed row missing projection")))

(:wat::core::defn :wat::query::sort-rows-by-sk
  [rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::into (:wat::query::empty-stored-rows)
    (:wat::core::sort-by :wat::query::StoredRow/sk (:wat::core::into [] rows))))

(:wat::core::defn :wat::query::sort-rows-by-isk
  [index <- :wat::core::String
   rows  <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::into (:wat::query::empty-stored-rows)
    (:wat::core::sort-by
      (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::core::String
        (:wat::query::row-isk index r))
      (:wat::core::into [] rows))))

;; insert AFTER equals. Append-fast-path when the new isk is last — queue send/receive
;; timestamps are monotonic, so the hot write does not rebuild the partition.
(:wat::core::defn :wat::query::insert-sorted-by-isk
  [index <- :wat::core::String
   rows  <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
   row   <- :wat::query::StoredRow]
  -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
  (:wat::core::let [isk (:wat::query::row-isk index row)]
    (:wat::core::if (:wat::core::= (:wat::core::count rows) 0)
      (:wat::vector::conj (:wat::query::empty-stored-rows) row)
      (:wat::core::if
        (:wat::core::>= isk
          (:wat::query::row-isk index
            (:wat::core::nth rows (:wat::core::- (:wat::core::count rows) 1))))
        (:wat::vector::conj rows row)
        (:wat::core::let
          [built (:wat::core::foldl
                   (:wat::core::fn [acc <- :wat::query::MemInsert x <- :wat::query::StoredRow]
                     -> :wat::query::MemInsert
                     (:wat::core::if (:wat::query::MemInsert/placed acc)
                       (:wat::query::MemInsert
                         :rows (:wat::vector::conj (:wat::query::MemInsert/rows acc) x)
                         :placed true)
                       (:wat::core::if (:wat::core::> (:wat::query::row-isk index x) isk)
                         (:wat::query::MemInsert
                           :rows (:wat::vector::conj
                                   (:wat::vector::conj (:wat::query::MemInsert/rows acc) row)
                                   x)
                           :placed true)
                         (:wat::query::MemInsert
                           :rows (:wat::vector::conj (:wat::query::MemInsert/rows acc) x)
                           :placed false))))
                   (:wat::query::MemInsert :rows (:wat::query::empty-stored-rows) :placed false)
                   rows)]
          (:wat::core::if (:wat::query::MemInsert/placed built)
            (:wat::query::MemInsert/rows built)
            (:wat::vector::conj (:wat::query::MemInsert/rows built) row)))))))

(:wat::core::defn :wat::query::index-drop-key
  [idx <- :wat::query::MemIndex k <- :wat::query::Key] -> :wat::query::MemIndex
  (:wat::core::let
    [pk (:wat::query::Key/pk k)
     sk (:wat::query::Key/sk k)]
    (:wat::core::match (:wat::query::live-get idx pk sk)
      (:wat::core::None idx)
      ((:wat::core::Some old)
        (:wat::core::let
          [idx1 (:wat::query::live-dissoc idx pk sk)
           idx2 (:wat::query::mem-set-pk-rows idx1 pk
                   (:wat::query::rows-without-key (:wat::query::mem-pk-rows idx1 pk) k))
           names (:wat::hashmap::keys (:wat::query::StoredRow/index-keys old))]
          (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::query::MemIndex name <- :wat::core::String]
              -> :wat::query::MemIndex
              (:wat::core::match (:wat::hashmap::get (:wat::query::StoredRow/index-keys old) name)
                (:wat::core::None acc)
                ((:wat::core::Some ik)
                  (:wat::core::let [ipk (:wat::query::IndexKey/ipk ik)]
                    (:wat::query::mem-set-gsi-rows acc name ipk
                      (:wat::query::rows-without-key (:wat::query::mem-gsi-rows acc name ipk) k))))))
            idx2
            names))))))

(:wat::core::defn :wat::query::index-add-row
  [idx <- :wat::query::MemIndex row <- :wat::query::StoredRow] -> :wat::query::MemIndex
  (:wat::core::let
    [pk (:wat::query::StoredRow/pk row)
     sk (:wat::query::StoredRow/sk row)
     idx1 (:wat::query::live-assoc idx pk sk row)
     idx2 (:wat::query::mem-mark-pk-dirty
            (:wat::query::mem-set-pk-rows idx1 pk
              (:wat::vector::conj (:wat::query::mem-pk-rows idx1 pk) row))
            pk)
     names (:wat::hashmap::keys (:wat::query::StoredRow/index-keys row))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::query::MemIndex name <- :wat::core::String]
        -> :wat::query::MemIndex
        (:wat::core::match (:wat::hashmap::get (:wat::query::StoredRow/index-keys row) name)
          (:wat::core::None acc)
          ((:wat::core::Some ik)
            (:wat::core::let [ipk (:wat::query::IndexKey/ipk ik)]
              (:wat::query::mem-set-gsi-rows acc name ipk
                (:wat::query::insert-sorted-by-isk name (:wat::query::mem-gsi-rows acc name ipk) row))))))
      idx2
      names)))

(:wat::core::defn :wat::query::index-put-row
  [idx <- :wat::query::MemIndex row <- :wat::query::StoredRow] -> :wat::query::MemIndex
  (:wat::query::index-add-row
    (:wat::query::index-drop-key idx
      (:wat::query::Key :pk (:wat::query::StoredRow/pk row) :sk (:wat::query::StoredRow/sk row)))
    row))

(:wat::core::defn :wat::query::ensure-pk-sorted
  [idx <- :wat::query::MemIndex pk <- :wat::core::String] -> :wat::query::MemIndex
  (:wat::core::match (:wat::hashmap::get (:wat::query::MemIndex/pk-dirty idx) pk)
    (:wat::core::None idx)
    ((:wat::core::Some dirty?)
      (:wat::core::if dirty?
        (:wat::query::mem-mark-pk-clean
          (:wat::query::mem-set-pk-rows idx pk
            (:wat::query::sort-rows-by-sk (:wat::query::mem-pk-rows idx pk)))
          pk)
        idx))))

(:wat::core::defn :wat::query::index-add-row-raw
  [idx <- :wat::query::MemIndex row <- :wat::query::StoredRow] -> :wat::query::MemIndex
  (:wat::core::let
    [pk (:wat::query::StoredRow/pk row)
     sk (:wat::query::StoredRow/sk row)
     idx1 (:wat::query::live-assoc idx pk sk row)
     idx2 (:wat::query::mem-set-pk-rows idx1 pk
             (:wat::vector::conj (:wat::query::mem-pk-rows idx1 pk) row))
     names (:wat::hashmap::keys (:wat::query::StoredRow/index-keys row))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::query::MemIndex name <- :wat::core::String]
        -> :wat::query::MemIndex
        (:wat::core::match (:wat::hashmap::get (:wat::query::StoredRow/index-keys row) name)
          (:wat::core::None acc)
          ((:wat::core::Some ik)
            (:wat::core::let [ipk (:wat::query::IndexKey/ipk ik)]
              (:wat::query::mem-set-gsi-rows acc name ipk
                (:wat::vector::conj (:wat::query::mem-gsi-rows acc name ipk) row))))))
      idx2
      names)))

(:wat::core::defn :wat::query::sort-all-partitions
  [idx <- :wat::query::MemIndex] -> :wat::query::MemIndex
  (:wat::core::let
    [idx1 (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::query::MemIndex pk <- :wat::core::String]
              -> :wat::query::MemIndex
              (:wat::query::mem-mark-pk-clean
                (:wat::query::mem-set-pk-rows acc pk
                  (:wat::query::sort-rows-by-sk (:wat::query::mem-pk-rows acc pk)))
                pk))
            idx
            (:wat::hashmap::keys (:wat::query::MemIndex/by-pk idx)))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::query::MemIndex name <- :wat::core::String]
        -> :wat::query::MemIndex
        (:wat::core::let
          [inner (:wat::core::Option/expect
                   (:wat::hashmap::get (:wat::query::MemIndex/by-index acc) name)
                   "mem-store: sort-all: index name present in keys")]
          (:wat::core::foldl
            (:wat::core::fn [acc2 <- :wat::query::MemIndex ipk <- :wat::core::String]
              -> :wat::query::MemIndex
              (:wat::query::mem-set-gsi-rows acc2 name ipk
                (:wat::query::sort-rows-by-isk name (:wat::query::mem-gsi-rows acc2 name ipk))))
            acc
            (:wat::hashmap::keys inner))))
      idx1
      (:wat::hashmap::keys (:wat::query::MemIndex/by-index idx1)))))

(:wat::core::defn :wat::query::rebuild-mem-index
  [rows <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])] -> :wat::query::MemIndex
  (:wat::query::sort-all-partitions
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::query::MemIndex r <- :wat::query::StoredRow] -> :wat::query::MemIndex
        (:wat::query::index-add-row-raw acc r))
      (:wat::query::empty-mem-index)
      rows)))

;; skip while sk < lo or not after cursor; then take while sk <= hi; then take lim.
;; Sorted, so this stops at hi / limit and does not walk the rest of the partition.
(:wat::core::defn :wat::query::take-base-page
  [rows   <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
   lo     <- :wat::core::String
   hi     <- :wat::core::String
   cursor <- (:wat::core::Option :- [:wat::core::String])
   lim    <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::query::Row])
  (:wat::core::into []
    (:wat::core::map :wat::query::StoredRow->Row
      (:wat::core::take
        (:wat::core::take-while
          (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::core::bool
            (:wat::core::<= (:wat::query::StoredRow/sk r) hi))
          (:wat::core::drop-while
            (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::core::bool
              (:wat::core::if (:wat::core::< (:wat::query::StoredRow/sk r) lo)
                true
                (:wat::core::if (:wat::query::sk-after-cursor? (:wat::query::StoredRow/sk r) cursor)
                  false
                  true)))
            rows))
        lim))))

(:wat::core::defn :wat::query::take-index-page
  [rows   <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
   index  <- :wat::core::String
   lo     <- :wat::core::String
   hi     <- :wat::core::String
   cursor <- (:wat::core::Option :- [:wat::core::String])
   lim    <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::query::IndexRow])
  (:wat::core::into []
    (:wat::core::map
      (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::query::IndexRow
        (:wat::query::StoredRow->IndexRow r
          (:wat::core::Option/expect
            (:wat::query::row-index-key r index)
            "mem-store: indexed partition row missing projection")))
      (:wat::core::take
        (:wat::core::take-while
          (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::core::bool
            (:wat::core::<= (:wat::query::row-isk index r) hi))
          (:wat::core::drop-while
            (:wat::core::fn [r <- :wat::query::StoredRow] -> :wat::core::bool
              (:wat::core::if (:wat::core::< (:wat::query::row-isk index r) lo)
                true
                (:wat::core::if (:wat::query::sk-after-cursor? (:wat::query::row-isk index r) cursor)
                  false
                  true)))
            rows))
        lim))))

;; ─── the mem-store' SERVICE — the real, mutating in-memory backend ──────────────────────────
;; durable = one flat (PersistentVector :- [StoredRow]); `put` is a replace-by-(pk,sk)
;; (DynamoDB PutItem — drop any existing row the incoming key names, then conj; later
;; rows in the batch win). The ephemeral MemIndex is derived at :init from that table
;; and maintained on put/delete. The `serve` loop rebinds `state` to the returned new State
;; (wat/service.wat's tail-recursive dispatch, `Outcome::Reply`); `scan`/`scan-index` are
;; pure reads (state unchanged) that lookup+range+take a partition. `:satisfies :wat::query::Store`
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
  :ephemeral [index <- :wat::query::MemIndex]
  :init (:wat::core::fn [record <- :wat::query::mem-store::Record]
          -> :wat::query::mem-store::State
          (:wat::query::mem-store::State
            :durable record
            :index (:wat::query::rebuild-mem-index (:wat::query::mem-store::Record/rows record))))
  :impls
  [(ensure-schema [s ctx req]
     ;; idempotent no-op — mem-store' has no physical schema to establish (the contract's
     ;; promise is satisfied trivially; sqlite's satisfier is where CREATE TABLE/INDEX happens).
     (:wat::service::Outcome::Reply s (:wat::query::Store::EnsureSchemaResponse::Success)))

   (put [s ctx req]
     ;; PutItem: for each incoming row, drop any acc row with the same (pk,sk)
     ;; (`key-hits-row?`, already the delete predicate) then conj. Later rows in
     ;; the batch win — the same sequential replace sqlite's put-rows recursion
     ;; produces. The ephemeral index is maintained the same way: drop the old
     ;; (pk,sk) (and its GSI projections) then insert the new row into the
     ;; sorted partitions.
     (:wat::core::let
       [new-rows (:wat::query::Store::PutRequest/rows req)
        dur (:wat::query::mem-store::State/durable s)
        merged (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                                  r   <- :wat::query::StoredRow]
                   -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                   (:wat::core::let
                     [k (:wat::query::Key :pk (:wat::query::StoredRow/pk r) :sk (:wat::query::StoredRow/sk r))
                      kept (:wat::core::foldl
                             (:wat::core::fn [a <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                                              x <- :wat::query::StoredRow]
                               -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                               (:wat::core::if (:wat::query::key-hits-row? k x)
                                 a
                                 (:wat::vector::conj a x)))
                             (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                             acc)]
                     (:wat::vector::conj kept r)))
                 (:wat::query::mem-store::Record/rows dur)
                 new-rows)
        idx (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::query::MemIndex r <- :wat::query::StoredRow]
                -> :wat::query::MemIndex
                (:wat::query::index-put-row acc r))
              (:wat::query::mem-store::State/index s)
              new-rows)]
       (:wat::service::Outcome::Reply
         (:wat::query::mem-store::State
           :durable (:wat::query::mem-store::Record merged)
           :index idx)
         (:wat::query::Store::PutResponse::Success))))

   (delete [s ctx req]
     ;; inverted put: fold the live table, keep rows whose (pk,sk) is not in the
     ;; key batch. A missing key is a no-op (the fold just keeps every row) —
     ;; PutResponse's arm list has no NotFound, so DeleteResponse cannot grow one;
     ;; SQL DELETE of 0 rows is the same Success. Report, do not invent an arm.
     ;; The ephemeral index drops the same keys.
     (:wat::core::let
       [keys (:wat::query::Store::DeleteRequest/keys req)
        dur (:wat::query::mem-store::State/durable s)
        kept (:wat::core::foldl
               (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                                r   <- :wat::query::StoredRow]
                 -> (:wat::core::PersistentVector :- [:wat::query::StoredRow])
                 (:wat::core::if (:wat::query::row-in-delete-batch? r keys)
                   acc
                   (:wat::vector::conj acc r)))
               (:wat::core::PersistentVector :- [:wat::query::StoredRow])
               (:wat::query::mem-store::Record/rows dur))
        idx (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::query::MemIndex k <- :wat::query::Key]
                -> :wat::query::MemIndex
                (:wat::query::index-drop-key acc k))
              (:wat::query::mem-store::State/index s)
              keys)]
       (:wat::service::Outcome::Reply
         (:wat::query::mem-store::State
           :durable (:wat::query::mem-store::Record kept)
           :index idx)
         (:wat::query::Store::DeleteResponse::Success))))

   (scan [s ctx req]
     (:wat::core::let
       [pk  (:wat::query::Store::ScanRequest/pk req)
        lo  (:wat::query::Store::ScanRequest/sk-lo req)
        hi  (:wat::query::Store::ScanRequest/sk-hi req)
        lim (:wat::query::Store::ScanRequest/limit req)
        cur (:wat::query::Store::ScanRequest/cursor req)
        idx (:wat::query::ensure-pk-sorted (:wat::query::mem-store::State/index s) pk)
        limited (:wat::query::take-base-page
                  (:wat::query::mem-pk-rows idx pk)
                  lo hi cur lim)
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::Row/sk (:wat::core::Option/expect (:wat::core::last limited) "scan: limited non-empty when full")))
                   :wat::core::None)
        s1 (:wat::query::mem-store::State
             :durable (:wat::query::mem-store::State/durable s)
             :index idx)]
       (:wat::service::Outcome::Reply s1 (:wat::query::Store::ScanResponse::Success limited next-cur))))

   (scan-index [s ctx req]
     (:wat::core::let
       [index (:wat::query::Store::ScanIndexRequest/index req)
        ipk   (:wat::query::Store::ScanIndexRequest/ipk req)
        lo    (:wat::query::Store::ScanIndexRequest/isk-lo req)
        hi    (:wat::query::Store::ScanIndexRequest/isk-hi req)
        lim   (:wat::query::Store::ScanIndexRequest/limit req)
        cur   (:wat::query::Store::ScanIndexRequest/cursor req)
        limited (:wat::query::take-index-page
                  (:wat::query::mem-gsi-rows (:wat::query::mem-store::State/index s) index ipk)
                  index lo hi cur lim)
        full?    (:wat::core::= (:wat::core::count limited) lim)
        next-cur (:wat::core::if full?
                   (:wat::core::Some (:wat::query::IndexRow/isk (:wat::core::Option/expect (:wat::core::last limited) "scan-index: limited non-empty when full")))
                   :wat::core::None)]
       (:wat::service::Outcome::Reply s (:wat::query::Store::ScanIndexResponse::Success limited next-cur))))])

;; probe-store-write-cost.wat — MEASURE the mem-store's WRITE cost, before fixing it.
;;
;; perf-2 made reads flat (119/116/123ms for 200 scans over 250/500/1000 rows). The circuit only
;; moved 287s -> 257s, and the reason named in SCORE-perf-2 is the write path: mem.wat:516-531,
;; `put` is a NESTED foldl -- for each incoming row it folds the ENTIRE table to rebuild `kept`,
;; then conj's. `delete` (:555) walks the table the same way.
;;
;; This measures that rather than asserting it. Time N puts into a FRESH store, for N, 2N, 4N.
;;   total ~2x per doubling => per-put cost is constant (what we want)
;;   total ~4x per doubling => per-put cost is O(table): the nested foldl, and O(n^2) overall
;;
;; It also times N deletes over a filled table, because the queue's `ack` is a delete and the
;; circuit does one per message.

(:wat::core::defn :wc::row [n <- :wat::core::i64] -> :wat::query::StoredRow
  (:wat::query::StoredRow
    :pk "q"
    :sk (:wat::core::format "{n}" :n n)
    :data "\"x\""
    :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])))

(:wat::core::defn :wc::put-n
  [c <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::query::Store/put c
              (:wat::query::Store::PutRequest
                :rows (:wat::core::Vector :- [:wat::query::StoredRow] (:wc::row n))))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil))]
      (:wc::put-n c (:wat::i64::- n 1)))))

(:wat::core::defn :wc::del-n
  [c <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::query::Store/delete c
              (:wat::query::Store::DeleteRequest
                :keys (:wat::core::Vector :- [:wat::query::Key]
                        (:wat::query::Key :pk "q" :sk (:wat::core::format "{n}" :n n)))))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil) (:wat::kernel::RecvOutcome::TimedOut nil))]
      (:wc::del-n c (:wat::i64::- n 1)))))

;; ⚠ the store handle is bound HERE, in the timing fn, never in a helper that returns the peer.
;; This file originally carried a `:wc::dial` helper doing exactly that, and the excursus-002
;; handle-lifetime wall rejected it — HandleCreationEscape, naming the /start span and the escape.
;; It was dead code I forgot to delete, and the wall built this morning caught it hours later on its
;; own author's file. Deleted rather than runed: a rune is for an instrument that must construct the
;; forbidden state, and this one simply had no reason to exist.
(:wat::core::defn :wc::time-puts [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c   (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr msh))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _p  (:wc::put-n c n)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     ms  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :wc::time-deletes [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c   (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr msh))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     _f  (:wc::put-n c n)
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _d  (:wc::del-n c n)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     ms  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p1 (:wc::time-puts 250)
     p2 (:wc::time-puts 500)
     p3 (:wc::time-puts 1000)
     d1 (:wc::time-deletes 250)
     d2 (:wc::time-deletes 500)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "puts 250/500/1000 -> {a}/{b}/{c}ms | deletes 250/500 -> {d}/{e}ms  (4x per doubling = O(table) per call)"
        :a p1 :b p2 :c p3 :d d1 :e d2))))

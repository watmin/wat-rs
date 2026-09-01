;; probe-store-scan-cost.wat — MEASURE the mem-store's read cost.
;;
;; 200 scans, limit 1: scan count and result size stay constant; only the table grows.
;; Baseline (unindexed walk): 1691 / 3489 / 9204 ms at 250 / 500 / 1000 rows.
;; After the index: cost per scan should be roughly flat across those sizes.
;;
;; scan-index is the hotter path (queue receive). Same shape, same sizes, reported separately.

(:wat::core::defn :sc::put-n
  [c <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= n 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::query::Store/put c
              (:wat::query::Store::PutRequest
                :rows (:wat::core::Vector :- [:wat::query::StoredRow]
                        (:wat::query::StoredRow
                          :pk "q"
                          :sk (:wat::core::format "{n}" :n n)
                          :data "\"x\""
                          :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                        "by-sk" (:wat::query::IndexKey
                                                  :ipk "q"
                                                  :isk (:wat::core::format "{n}" :n n)))))))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil))]
      (:sc::put-n c (:wat::i64::- n 1)))))

;; k scans, each with limit 1 — the RESULT size is constant regardless of table size.
(:wat::core::defn :sc::scan-k
  [c <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   k <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= k 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::query::Store/scan c
              (:wat::query::Store::ScanRequest
                :pk "q" :sk-lo "" :sk-hi "zzzz" :limit 1 :cursor :wat::core::None))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil))]
      (:sc::scan-k c (:wat::i64::- k 1)))))

(:wat::core::defn :sc::scan-index-k
  [c <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   k <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::if (:wat::i64::<= k 0)
    nil
    (:wat::core::let
      [_r (:wat::core::match
            (:wat::query::Store/scan-index c
              (:wat::query::Store::ScanIndexRequest
                :index "by-sk" :ipk "q" :isk-lo "" :isk-hi "zzzz" :limit 1 :cursor :wat::core::None))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil))]
      (:sc::scan-index-k c (:wat::i64::- k 1)))))

;; time `scans` reads against a table of `rows`. Returns elapsed ms for the SCANS only.
(:wat::core::defn :sc::time-scans [rows <- :wat::core::i64  scans <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c   (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr msh))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     _p  (:sc::put-n c rows)
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _s  (:sc::scan-k c scans)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     ms  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :sc::time-scan-indexes [rows <- :wat::core::i64  scans <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c   (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr msh))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     _p  (:sc::put-n c rows)
     t0  (:wat::time::epoch-nanos (:wat::time::now))
     _s  (:sc::scan-index-k c scans)
     t1  (:wat::time::epoch-nanos (:wat::time::now))
     ms  (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)]
    ms))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:sc::time-scans 250 200)
     b (:sc::time-scans 500 200)
     c (:sc::time-scans 1000 200)
     d (:sc::time-scan-indexes 250 200)
     e (:sc::time-scan-indexes 500 200)
     f (:sc::time-scan-indexes 1000 200)]
    (:wat::kernel::println
      (:wat::string::interpolate
        "200 scans (limit 1) over table=250/500/1000 -> scan={x}/{y}/{z}ms scan-index={u}/{v}/{w}ms"
        :x a :y b :z c :u d :v e :w f))))

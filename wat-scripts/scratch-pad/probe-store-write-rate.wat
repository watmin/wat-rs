;; probe-store-write-rate.wat — mem-store vs sqlite-store, WRITE rate, at process locus.
;;
;; WHY. The circuit's drain is 70.9s of an 85.5s wall — 35 ms per message, and every message is
;; four adapter->queue->store chains. The only recorded comparison (SCORE-perf-2:51, "sqlite runs
;; the same circuit in 43 s" against mem's 257 s) was taken BEFORE perf-3 rebuilt mem's write path
;; and took the circuit to 88.6 s. Quoting that 6x today would be a measurement of one thing
;; asserted about another. This takes a current one, one variable: the backend.
;;
;; Same row shape the queue writes (sqs.wat:216-222): pk=queue, sk=uuid, one index key.
;; Same locus the circuit uses (process). Same N as the circuit's total deliveries.

(:wat::core::defn :probe::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dial-store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::put-one
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   i     <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [row (:wat::query::StoredRow
           :pk "q" :sk (:wat::edn::write (:wat::uuid::v4)) :data (:wat::core::str i)
           :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                         "by-visible-at" (:wat::query::IndexKey :ipk "q" :isk (:wat::core::str i))))]
    (:wat::core::match
      (:wat::query::Store/put store
        (:wat::query::Store::PutRequest (:wat::core::Vector :- [:wat::query::StoredRow] row)))
      ((:wat::kernel::RecvOutcome::Message r)
        (:wat::core::match r
          ((:wat::query::Store::PutResponse::Success) nil)
          (_ (:wat::kernel::assertion-failed! "put not Success" :wat::core::None :wat::core::None))))
      (_ (:wat::kernel::assertion-failed! "put recv failed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :probe::drive
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n     <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [_es (:wat::core::match
           (:wat::query::Store/ensure-schema store
             (:wat::query::Store::EnsureSchemaRequest
               :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
               :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                          (:wat::query::IndexSchema
                            :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
           ((:wat::kernel::RecvOutcome::Message _r) nil)
           (_ (:wat::kernel::assertion-failed! "ensure-schema failed" :wat::core::None :wat::core::None)))
     t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
            (:probe::put-one store i))
          nil
          (:wat::core::range 0 n))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) 1000000)))

(:wat::core::defn :probe::mem-ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::query::mem-store/start :locus (:wat::spawn::process)
         :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c (:probe::dial-store (:wat::query::mem-store::Handle/addr h))
     ms (:probe::drive c n)]
    ms))

(:wat::core::defn :probe::sqlite-ms [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [h (:wat::query::sqlite-store/start :locus (:wat::spawn::process)
         :record (:wat::query::sqlite-store::Record
                   :path ":memory:"
                   :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     c (:probe::dial-store (:wat::query::sqlite-store::Handle/addr h))
     ms (:probe::drive c n)]
    ms))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "mem={m}ms;sqlite={s}ms" :m (:probe::mem-ms 200) :s (:probe::sqlite-ms 200)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format
      "mem: 1000={a} 2000={b} 4000={c} || sqlite: 1000={d} 2000={e} 4000={f}"
      :a (:probe::mem-ms 1000) :b (:probe::mem-ms 2000) :c (:probe::mem-ms 4000)
      :d (:probe::sqlite-ms 1000) :e (:probe::sqlite-ms 2000) :f (:probe::sqlite-ms 4000))))

;; probe-a-count-never-reads-more-than-it-needs.wat
;;
;; CountIndexRequest.limit saturates. 5000 rows: limit 65 → 65, limit 100000 → 5000.
;; mem and sqlite agree. Queue cap+1 still rejects a send at cap.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :cn::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "cn: dial-store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cn::ensure
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema st
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                   (:wat::query::IndexSchema
                     :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "cn: ensure-schema failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :cn::put-n
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n  <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [batch 50
     nbatch (:wat::i64::/ (:wat::i64::+ n (:wat::i64::- batch 1)) batch)]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  b <- :wat::core::i64] -> :wat::core::nil
        (:wat::core::let
          [start (:wat::i64::* b batch)
           ntake (:wat::core::if (:wat::i64::>= (:wat::i64::+ start batch) n)
                    (:wat::i64::- n start)
                    batch)
           rows (:wat::core::foldl
                  (:wat::core::fn [rs <- (:wat::core::Vector :- [:wat::query::StoredRow])  i <- :wat::core::i64]
                    -> (:wat::core::Vector :- [:wat::query::StoredRow])
                    (:wat::core::let
                      [k (:wat::core::format "{n}" :n (:wat::i64::+ 10000 (:wat::i64::+ start i)))]
                      (:wat::core::conj rs
                        (:wat::query::StoredRow
                          :pk "t" :sk k :data k
                          :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                        "by-visible-at" (:wat::query::IndexKey :ipk "idx" :isk k))))))
                  (:wat::core::Vector :- [:wat::query::StoredRow])
                  (:wat::core::range 0 ntake))]
          (:wat::core::match
            (:wat::query::Store/put st (:wat::query::Store::PutRequest :rows rows))
            ((:wat::kernel::RecvOutcome::Message r)
              (:wat::core::match r
                ((:wat::query::Store::PutResponse::Success) nil)
                (_ (:wat::kernel::assertion-failed! "cn: put not Success" :wat::core::None :wat::core::None))))
            (_ (:wat::kernel::assertion-failed! "cn: put recv failed" :wat::core::None :wat::core::None)))))
      nil
      (:wat::core::range 0 nbatch))))

(:wat::core::defn :cn::count-at
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   lim <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/count-index st
      (:wat::query::Store::CountIndexRequest
        :index "by-visible-at" :ipk "idx"
        :isk-lo "0" :isk-hi "z" :limit lim))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::CountIndexResponse::Ok n) n)
        (_ -1)))
    (_ -2)))

(:wat::core::defn :cn::fill-store
  [st <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:cn::ensure st)
     _ (:cn::put-n st 5000)]
    nil))

(:wat::core::defn :cn::send-one
  [q <- :queue::Queue  i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
    (:queue::Queue/send q
      (:queue::Queue::SendRequest
        :queue "q"
        :bodies (:wat::core::Vector :- [:wat::core::String] (:wat::core::format "b{i}" :i i))
        :now-ns (:wat::i64::+ 1000 i)))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::SendResponse::Accepted n) n)
        (_ -1)))
    (_ -2)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     mst (:cn::dial-store (:wat::query::mem-store::Handle/addr msh))
     _ (:cn::fill-store mst)
     m65 (:cn::count-at mst 65)
     m5k (:cn::count-at mst 100000)
     m1  (:cn::count-at mst 1)
     ssh (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::sqlite-store::Record
                     :path ":memory:"
                     :index-names (:wat::core::Vector :- [:wat::core::String] "by-visible-at")))
     sst (:cn::dial-store (:wat::query::sqlite-store::Handle/addr ssh))
     _ (:cn::fill-store sst)
     s65 (:cn::count-at sst 65)
     s5k (:cn::count-at sst 100000)
     s1  (:cn::count-at sst 1)
     qsh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 4
                    :store-addr (:wat::query::mem-store::Handle/addr qsh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q (:wat::core::match (:wat::kernel::connect (:queue::queue::Handle/addr qh))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "cn: dial-queue failed" :wat::core::None :wat::core::None)))
     a0 (:cn::send-one q 0)
     a1 (:cn::send-one q 1)
     a2 (:cn::send-one q 2)
     a3 (:cn::send-one q 3)
     a4 (:cn::send-one q 4)]
    (:wat::core::format
      "mem65={m65};mem5000={m5k};mem1={m1};sql65={s65};sql5000={s5k};sql1={s1};agree={ag};fill={f};over={o}"
      :m65 m65 :m5k m5k :m1 m1 :s65 s65 :s5k s5k :s1 s1
      :ag (:wat::core::if (:wat::core::and
                            (:wat::core::and (:wat::core::= m65 s65) (:wat::core::= m5k s5k))
                            (:wat::core::= m1 s1))
            "yes" "no")
      :f (:wat::core::format "{a}/{b}/{c}/{d}" :a a0 :b a1 :c a2 :d a3)
      :o a4)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))

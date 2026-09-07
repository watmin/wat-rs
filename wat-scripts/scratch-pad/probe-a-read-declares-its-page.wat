;; probe-a-read-declares-its-page.wat
;;
;; :max-page truncates; scan-index-all yields every row in order; receive is
;; bounded and has no -all. limit is a request, not a promise.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :ch::dial-store
  [a <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected c) c)
    (_ (:wat::kernel::assertion-failed! "dial-store failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :ch::put-n
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   n     <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::core::match
         (:wat::query::Store/ensure-schema store
           (:wat::query::Store::EnsureSchemaRequest
             :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
             :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                        (:wat::query::IndexSchema
                          :name "by-visible-at" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
         ((:wat::kernel::RecvOutcome::Message _r) nil)
         (_ (:wat::kernel::assertion-failed! "ensure-schema failed" :wat::core::None :wat::core::None)))]
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::nil  i <- :wat::core::i64] -> :wat::core::nil
        (:wat::core::let
          [isk (:wat::core::format "{n}" :n (:wat::i64::+ 10000 i))
           row (:wat::query::StoredRow
                 :pk "t" :sk isk :data isk
                 :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                               "by-visible-at" (:wat::query::IndexKey :ipk "idx" :isk isk)))]
          (:wat::core::match
            (:wat::query::Store/put store
              (:wat::query::Store::PutRequest
                :rows (:wat::core::Vector :- [:wat::query::StoredRow] row)))
            ((:wat::kernel::RecvOutcome::Message r)
              (:wat::core::match r
                ((:wat::query::Store::PutResponse::Success) nil)
                (_ (:wat::kernel::assertion-failed! "put not Success" :wat::core::None :wat::core::None))))
            (_ (:wat::kernel::assertion-failed! "put recv failed" :wat::core::None :wat::core::None)))))
      nil
      (:wat::core::range 0 n))))

(:wat::core::defn :ch::scan-page
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::match
    (:wat::query::Store/scan-index store
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk "idx"
        :isk-lo "0" :isk-hi "z" :limit 1000 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::ScanIndexResponse::Success rows cur)
          (:wat::core::Tuple (:wat::core::count rows)
            (:wat::core::match cur
              ((:wat::core::Some _k) true)
              (:wat::core::None false))))
        (_ (:wat::core::Tuple -1 false))))
    (_ (:wat::core::Tuple -2 false))))

(:wat::core::defn :ch::drain-all
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> (:wat::core::Vector :- [:wat::query::IndexRow])
  (:wat::core::into
    (:wat::core::Vector :- [:wat::query::IndexRow])
    (:wat::query::Store::scan-index-all store
      (:wat::query::Store::ScanIndexRequest
        :index "by-visible-at" :ipk "idx"
        :isk-lo "0" :isk-hi "z" :limit 1000 :cursor :wat::core::None))))

(:wat::core::defn :ch::ordered?
  [rows <- (:wat::core::Vector :- [:wat::query::IndexRow])] -> :wat::core::bool
  (:wat::core::let
    [n (:wat::core::count rows)]
    (:wat::core::foldl
      (:wat::core::fn [ok <- :wat::core::bool  i <- :wat::core::i64] -> :wat::core::bool
        (:wat::core::if (:wat::core::not ok)
          ok
          (:wat::core::if (:wat::core::= i 0)
            true
            (:wat::core::<= (:wat::query::IndexRow/isk (:wat::core::nth rows (:wat::i64::- i 1)))
                            (:wat::query::IndexRow/isk (:wat::core::nth rows i))))))
      true
      (:wat::core::range 0 n))))

(:wat::core::defn :ch::take-first
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::i64
  (:wat::core::count
    (:wat::core::into
      (:wat::core::Vector :- [:wat::query::IndexRow])
      (:wat::core::take
        (:wat::query::Store::scan-index-all store
          (:wat::query::Store::ScanIndexRequest
            :index "by-visible-at" :ipk "idx"
            :isk-lo "0" :isk-hi "z" :limit 1000 :cursor :wat::core::None))
        1))))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [now (:wat::time::epoch-nanos (:wat::time::now))
     msh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
           :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     st (:ch::dial-store (:wat::query::mem-store::Handle/addr msh))
     _ (:ch::put-n st 250)
     page (:ch::scan-page st)
     pn (:wat::core::first page)
     some-cur? (:wat::core::second page)
     all (:ch::drain-all st)
     an (:wat::core::count all)
     ord? (:ch::ordered? all)
     first-n (:ch::take-first st)
     pages (:wat::i64::/ (:wat::i64::+ 250 63) 64)
     msh2 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 1000
                    :store-addr (:wat::query::mem-store::Handle/addr msh2)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q (:user::dial-queue (:queue::queue::Handle/addr qh))
     stg (:wat::core::format "Accepted({n})"
            :n
            (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::match
                  (:queue::Queue/send q
                    (:queue::Queue::SendRequest
                      :queue "q"
                      :bodies (:wat::core::Vector :- [:wat::core::String] (:wat::core::format "b{i}" :i i))
                      :now-ns (:wat::i64::+ now i)))
                  ((:wat::kernel::RecvOutcome::Message r)
                    (:wat::core::match r
                      ((:queue::Queue::SendResponse::Accepted n) (:wat::i64::+ acc n))
                      (_ acc)))
                  (_ acc)))
              0
              (:wat::core::range 0 70)))
     rec (:queue::Queue/receive q
           (:queue::Queue::ReceiveRequest
             :queue "q" :now-ns (:wat::i64::+ now 1000) :visibility-ns 1000000000000
             :limit 1000 :wait (:queue::Queue::Wait::Immediate)))
     rn (:wat::core::match rec
          ((:wat::kernel::RecvOutcome::Message r)
            (:wat::core::match r
              ((:queue::Queue::ReceiveResponse::Ok envs) (:wat::core::count envs))
              (_ -1)))
          (_ -2))]
    (:wat::core::format
      "page-n={pn};some-cur={sc};all-n={an};ordered={o};first-n={fn};pages={pg};send={stg};recv-n={rn};recv-max={rm};field={f}"
      :pn pn
      :sc (:wat::core::if some-cur? "yes" "no")
      :an an
      :o (:wat::core::if ord? "yes" "no")
      :fn first-n
      :pg pages
      :stg stg
      :rn rn
      :rm :queue::Queue::RECEIVE-MAX-PAGE
      :f :queue::Queue::RECEIVE-MAX-PAGE-FIELD)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))

;; probe-store-hibernate-index.wat — resume rebuilds the ephemeral index from durable.
;;
;; Put two rows (one GSI), scan + scan-index, hibernate, resume from the Record, scan
;; + scan-index again. The pages must match. The index is :ephemeral — if :init did
;; not rebuild it, the resumed store would answer empty.

(:wat::core::defn :hi::dial
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))))

(:wat::core::defn :hi::put
  [c <- :wat::query::Store] -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/put c
      (:wat::query::Store::PutRequest
        :rows (:wat::core::Vector :- [:wat::query::StoredRow]
                (:wat::query::StoredRow
                  :pk "q" :sk "a" :data "1"
                  :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                "by-v" (:wat::query::IndexKey :ipk "q" :isk "v1")))
                (:wat::query::StoredRow
                  :pk "q" :sk "b" :data "2"
                  :index-keys (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
                                "by-v" (:wat::query::IndexKey :ipk "q" :isk "v2"))))))
    ((:wat::kernel::RecvOutcome::Message _r) nil)
    (_ (:wat::kernel::assertion-failed! "hi.put failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :hi::render
  [c <- :wat::query::Store] -> :wat::core::String
  (:wat::core::let
    [base (:wat::core::match
            (:wat::query::Store/scan c
              (:wat::query::Store::ScanRequest :pk "q" :sk-lo "" :sk-hi "z" :limit 10 :cursor :wat::core::None))
            ((:wat::kernel::RecvOutcome::Message r)
              (:wat::core::match r
                ((:wat::query::Store::ScanResponse::Success rows _c)
                  (:wat::i64::to-string (:wat::core::count rows)))
                (_ "FAIL")))
            (_ "FAIL"))
     gsi (:wat::core::match
           (:wat::query::Store/scan-index c
             (:wat::query::Store::ScanIndexRequest
               :index "by-v" :ipk "q" :isk-lo "v0" :isk-hi "vz" :limit 10 :cursor :wat::core::None))
           ((:wat::kernel::RecvOutcome::Message r)
             (:wat::core::match r
               ((:wat::query::Store::ScanIndexResponse::Success rows _c)
                 (:wat::i64::to-string (:wat::core::count rows)))
               (_ "FAIL")))
           (_ "FAIL"))]
    (:wat::string::interpolate "base={b};gsi={g}" :b base :g gsi)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h1 (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     c1 (:hi::dial (:wat::query::mem-store::Handle/addr h1))
     _p (:hi::put c1)
     before (:hi::render c1)
     snap (:wat::query::mem-store/hibernate h1)
     h2 (:wat::query::mem-store/resume :locus (:wat::spawn::thread) :record snap)
     c2 (:hi::dial (:wat::query::mem-store::Handle/addr h2))
     after (:hi::render c2)]
    (:wat::kernel::println
      (:wat::string::interpolate "before={b} after={a} match={m}"
        :b before :a after :m (:wat::core::= before after)))))

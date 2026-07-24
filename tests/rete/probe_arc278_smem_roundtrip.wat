;; Co-located fixture for probe_arc278_smem_roundtrip.rs — arc 278 stone S-mem.gate acceptance gate.
;;
;; Proves the baked :wat::query::mem-store' (a real :wat::service::defservice `:satisfies
;; :wat::query::Store` satisfier, wat/query/mem.wat, on the services-as-surfaces OPERATION MODEL)
;; round-trips put -> scan -> keyset-paginate -> scan-index against the REAL backend. The dialed
;; peer IS the Store, intrinsically (arc 293 Path B) — no wrapper struct, no extend-type. Every op
;; returns a per-op OUTCOME ENUM (`Store::<Op>Response`, `:Success` first); mem-store' never
;; errors, so every response is matched to its `:Success` arm.

(:wat::test::deftest :user::smem_roundtrip 
  (:wat::core::let
    [h          (:wat::query::mem-store/start :locus (:wat::spawn::thread)
                  :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store      (:wat::core::match (:wat::kernel::connect' (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     empty-ik   (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
     ik-a       (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey :ipk "u#1" :isk "v1"))
     ik-c       (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey :ipk "u#1" :isk "v2"))
     input-rows (:wat::core::Vector :wat::query::StoredRow
                  (:wat::query::StoredRow :pk "u#1" :sk "a" :data "{:v 1}" :index-keys ik-a)
                  (:wat::query::StoredRow :pk "u#1" :sk "b" :data "{:v 2}" :index-keys empty-ik)
                  (:wat::query::StoredRow :pk "u#1" :sk "c" :data "{:v 3}" :index-keys ik-c)
                  (:wat::query::StoredRow :pk "u#1" :sk "d" :data "{:v 4}" :index-keys empty-ik)
                  (:wat::query::StoredRow :pk "u#1" :sk "e" :data "{:v 5}" :index-keys empty-ik))

     es-resp    (:wat::query::Store/ensure-schema store
                  (:wat::query::Store::EnsureSchemaRequest
                    :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
                    :indexes (:wat::core::Vector :wat::query::IndexSchema (:wat::query::IndexSchema :name "by-v" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
     put-resp   (:wat::query::Store/put store (:wat::query::Store::PutRequest input-rows))

     page1-resp (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor :wat::core::None))
     page2-resp (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor (:wat::core::Some "b")))
     page3-resp (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor (:wat::core::Some "d")))

     ipage-resp (:wat::query::Store/scan-index store
                  (:wat::query::Store::ScanIndexRequest
                    :index "by-v" :ipk "u#1" :isk-lo "v1" :isk-hi "v2" :limit 10 :cursor :wat::core::None))]

    (:wat::core::match es-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
      (_ (:wat::kernel::assertion-failed! "ensure-schema failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

    (:wat::core::match put-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::PutResponse::Success) nil)
      (_ (:wat::kernel::assertion-failed! "put failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

    (:wat::core::match page1-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "a")
          (:wat::test::assert-eq cursor (:wat::core::Some "b"))))
      (_ (:wat::kernel::assertion-failed! "scan page1 failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

    (:wat::core::match page2-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "c")
          (:wat::test::assert-eq cursor (:wat::core::Some "d"))))
      (_ (:wat::kernel::assertion-failed! "scan page2 failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

    (:wat::core::match page3-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 1)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "e")
          (:wat::test::assert-eq cursor :wat::core::None)))
      (_ (:wat::kernel::assertion-failed! "scan page3 failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

    (:wat::core::match ipage-resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
      ((:wat::query::Store::ScanIndexResponse::Success rows _cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::IndexRow/isk (:wat::core::first rows)) "v1")))
      (_ (:wat::kernel::assertion-failed! "scan-index failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))

;; Co-located fixture for probe_arc278_smem_roundtrip.rs — arc 278 stone S-mem.gate acceptance gate.
;;
;; Proves the baked :wat::query::mem-store' (a real :wat::service::defservice `:satisfies
;; :wat::query::Store` satisfier, wat/query/mem.wat, on the services-as-surfaces OPERATION MODEL)
;; round-trips put -> scan -> keyset-paginate -> scan-index against the REAL backend. The dialed
;; peer IS the Store, intrinsically (arc 293 Path B) — no wrapper struct, no extend-type. Every op
;; returns a per-op OUTCOME ENUM (`Store::<Op>Response`, `:Success` first); mem-store' never
;; errors, so every response is matched to its `:Success` arm.

(:wat::test::deftest' :user::smem_roundtrip ()
  (:wat::core::let
    [h          (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
                  :record (:wat::query::mem-store'::Record (:wat::core::PersistentVector)))
     store      (:wat::kernel::connect' (:wat::query::mem-store'::Handle/addr h))
     empty-ik   (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
     ik-a       (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey "u#1" "v1"))
     ik-c       (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey "u#1" "v2"))
     input-rows (:wat::core::Vector :wat::query::StoredRow
                  (:wat::query::StoredRow "u#1" "a" "{:v 1}" ik-a)
                  (:wat::query::StoredRow "u#1" "b" "{:v 2}" empty-ik)
                  (:wat::query::StoredRow "u#1" "c" "{:v 3}" ik-c)
                  (:wat::query::StoredRow "u#1" "d" "{:v 4}" empty-ik)
                  (:wat::query::StoredRow "u#1" "e" "{:v 5}" empty-ik))

     es-resp    (:wat::query::Store/ensure-schema store
                  (:wat::query::Store::EnsureSchemaRequest (:wat::query::TableSchema "pk" "sk")
                    (:wat::core::Vector :wat::query::IndexSchema (:wat::query::IndexSchema "by-v" "pk" "sk" "ipk" "isk"))))
     put-resp   (:wat::query::Store/put store (:wat::query::Store::PutRequest input-rows))

     page1-resp (:wat::query::Store/scan store (:wat::query::Store::ScanRequest "u#1" "a" "z" 2 :wat::core::None))
     page2-resp (:wat::query::Store/scan store (:wat::query::Store::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "b")))
     page3-resp (:wat::query::Store/scan store (:wat::query::Store::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "d")))

     ipage-resp (:wat::query::Store/scan-index store
                  (:wat::query::Store::ScanIndexRequest "by-v" "u#1" "v1" "v2" 10 :wat::core::None))]

    (:wat::core::match es-resp -> :wat::core::nil
      ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
      (_ (:wat::kernel::assertion-failed! "ensure-schema failed" :wat::core::None :wat::core::None)))

    (:wat::core::match put-resp -> :wat::core::nil
      ((:wat::query::Store::PutResponse::Success) nil)
      (_ (:wat::kernel::assertion-failed! "put failed" :wat::core::None :wat::core::None)))

    (:wat::core::match page1-resp -> :wat::core::nil
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "a")
          (:wat::test::assert-eq cursor (:wat::core::Some "b"))))
      (_ (:wat::kernel::assertion-failed! "scan page1 failed" :wat::core::None :wat::core::None)))

    (:wat::core::match page2-resp -> :wat::core::nil
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "c")
          (:wat::test::assert-eq cursor (:wat::core::Some "d"))))
      (_ (:wat::kernel::assertion-failed! "scan page2 failed" :wat::core::None :wat::core::None)))

    (:wat::core::match page3-resp -> :wat::core::nil
      ((:wat::query::Store::ScanResponse::Success rows cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 1)
          (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first rows)) "e")
          (:wat::test::assert-eq cursor :wat::core::None)))
      (_ (:wat::kernel::assertion-failed! "scan page3 failed" :wat::core::None :wat::core::None)))

    (:wat::core::match ipage-resp -> :wat::core::nil
      ((:wat::query::Store::ScanIndexResponse::Success rows _cursor)
        (:wat::core::do
          (:wat::test::assert-eq (:wat::core::count rows) 2)
          (:wat::test::assert-eq (:wat::query::IndexRow/isk (:wat::core::first rows)) "v1")))
      (_ (:wat::kernel::assertion-failed! "scan-index failed" :wat::core::None :wat::core::None)))))

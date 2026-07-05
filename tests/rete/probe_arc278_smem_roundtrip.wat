;; Co-located fixture for probe_arc278_smem_roundtrip.rs — arc 278 stone S-mem.gate acceptance gate.
;;
;; Proves the baked :wat::query::MemStore (a real :wat::service::defservice-backed Store/ReadStore
;; satisfier, wat/query/mem.wat) round-trips put -> scan -> keyset-paginate -> scan-index against
;; the REAL backend. S0's probe_arc278_query_contract.wat used an in-file stub returning empty
;; pages; this drives the actual spawned service through start/connect'/MemStore and asserts real
;; data flows through it.

(:wat::test::deftest' :user::smem_roundtrip ()
  (:wat::core::let
    [h        (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
                :record (:wat::query::mem-store'::Record (:wat::core::PersistentVector)))
     c        (:wat::kernel::connect' (:wat::query::mem-store'::Handle/addr h))
     store    (:wat::query::MemStore c)
     empty-ik (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
     ik-a     (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey "u#1" "v1"))
     ik-c     (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey "u#1" "v2"))
     rows     (:wat::core::Vector :wat::query::StoredRow
                (:wat::query::StoredRow "u#1" "a" "{:v 1}" ik-a)
                (:wat::query::StoredRow "u#1" "b" "{:v 2}" empty-ik)
                (:wat::query::StoredRow "u#1" "c" "{:v 3}" ik-c)
                (:wat::query::StoredRow "u#1" "d" "{:v 4}" empty-ik)
                (:wat::query::StoredRow "u#1" "e" "{:v 5}" empty-ik))
     _es      (:wat::core::Result/expect
                (:wat::query::Store/ensure-schema store (:wat::query::TableSchema "pk" "sk")
                  (:wat::core::Vector :wat::query::IndexSchema (:wat::query::IndexSchema "pk" "sk" "ipk" "isk")))
                "ensure-schema failed")
     _p       (:wat::core::Result/expect (:wat::query::Store/put store rows) "put failed")

     page1    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 :wat::core::None))
                "scan page1 failed")
     p1-rows  (:wat::query::Page/rows page1)

     page2    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "b")))
                "scan page2 failed")
     p2-rows  (:wat::query::Page/rows page2)

     page3    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "d")))
                "scan page3 failed")
     p3-rows  (:wat::query::Page/rows page3)

     ipage    (:wat::core::Result/expect
                (:wat::query::Store/scan-index store
                  (:wat::query::IndexScanRequest "by-v" "u#1" "v1" "v2" 10 :wat::core::None))
                "scan-index failed")
     i-rows   (:wat::query::IndexPage/rows ipage)]

    (:wat::test::assert-eq (:wat::core::count p1-rows) 2)
    (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first p1-rows)) "a")
    (:wat::test::assert-eq (:wat::query::Page/next-cursor page1) (:wat::core::Some "b"))

    (:wat::test::assert-eq (:wat::core::count p2-rows) 2)
    (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first p2-rows)) "c")
    (:wat::test::assert-eq (:wat::query::Page/next-cursor page2) (:wat::core::Some "d"))

    (:wat::test::assert-eq (:wat::core::count p3-rows) 1)
    (:wat::test::assert-eq (:wat::query::Row/sk (:wat::core::first p3-rows)) "e")
    (:wat::test::assert-eq (:wat::query::Page/next-cursor page3) :wat::core::None)

    (:wat::test::assert-eq (:wat::core::count i-rows) 2)
    (:wat::test::assert-eq (:wat::query::IndexRow/isk (:wat::core::first i-rows)) "v1")))

;; Co-located fixture for probe_arc278_sqlite_store_differential.rs — arc 278 stone S2 the
;; DIFFERENTIAL acceptance gate: the sqlite `:wat::query::Store` satisfier (`SqliteStore`,
;; wat/query/sqlite_store.wat) held bit-for-bit against the S-mem MemStore oracle
;; (wat/query/mem.wat, proven by probe_arc278_smem_roundtrip.wat).
;;
;; ONE `run-ops` fn drives the op sequence (mirroring the S-mem gate exactly: ensure-schema with
;; one GSI "by-v" -> put 5 rows on pk "u#1" [2 projecting the GSI] -> keyset-paginate scan 2/2/1 ->
;; scan-index) through the SURFACE `:wat::query::Store` type, so the SAME code exercises BOTH
;; backends. The gate constructs a MemStore AND a SqliteStore (":memory:"), runs `run-ops` on
;; EACH, and asserts the returned Pages are EQUAL — same ops -> same Pages, bit-for-bit.

(:wat::core::defrecord :probe::RunResult
  [page1 <- :wat::query::Page
   page2 <- :wat::query::Page
   page3 <- :wat::query::Page
   ipage <- :wat::query::IndexPage])

(:wat::core::defn :probe::run-ops [store <- :wat::query::Store] -> :probe::RunResult
  (:wat::core::let
    [empty-ik (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
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
                  (:wat::core::Vector :wat::query::IndexSchema (:wat::query::IndexSchema "by-v" "pk" "sk" "ipk" "isk")))
                "ensure-schema failed")
     _p       (:wat::core::Result/expect (:wat::query::Store/put store rows) "put failed")

     page1    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 :wat::core::None))
                "scan page1 failed")
     page2    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "b")))
                "scan page2 failed")
     page3    (:wat::core::Result/expect
                (:wat::query::Store/scan store (:wat::query::ScanRequest "u#1" "a" "z" 2 (:wat::core::Some "d")))
                "scan page3 failed")
     ipage    (:wat::core::Result/expect
                (:wat::query::Store/scan-index store
                  (:wat::query::IndexScanRequest "by-v" "u#1" "v1" "v2" 10 :wat::core::None))
                "scan-index failed")]
    (:probe::RunResult page1 page2 page3 ipage)))

(:wat::test::deftest' :user::sqlite_store_differential ()
  (:wat::core::let
    [h            (:wat::query::mem-store'/start :locus (:wat::spawn::thread)
                    :record (:wat::query::mem-store'::Record (:wat::core::PersistentVector)))
     c            (:wat::kernel::connect' (:wat::query::mem-store'::Handle/addr h))
     mem-store    (:wat::query::MemStore c)

     ;; T1a: the sqlite backend is now a defservice + peer-wrapping satisfier (mirroring MemStore's
     ;; construction above) — start the service INLINE (scope law), connect', wrap the peer. The
     ;; durable Record carries the path (":memory:") + the declared GSI-name set ("by-v").
     sh           (:wat::query::sqlite-store'/start :locus (:wat::spawn::thread)
                    :record (:wat::query::sqlite-store'::Record ":memory:"
                              (:wat::core::Vector :wat::core::String "by-v")))
     sc           (:wat::kernel::connect' (:wat::query::sqlite-store'::Handle/addr sh))
     sqlite-store (:wat::query::SqliteStore sc)

     mem-result    (:probe::run-ops mem-store)
     sqlite-result (:probe::run-ops sqlite-store)]

    ;; the differential: same ops through both backends -> same Pages, bit-for-bit.
    (:wat::test::assert-eq (:probe::RunResult/page1 mem-result) (:probe::RunResult/page1 sqlite-result))
    (:wat::test::assert-eq (:probe::RunResult/page2 mem-result) (:probe::RunResult/page2 sqlite-result))
    (:wat::test::assert-eq (:probe::RunResult/page3 mem-result) (:probe::RunResult/page3 sqlite-result))
    (:wat::test::assert-eq (:probe::RunResult/ipage mem-result) (:probe::RunResult/ipage sqlite-result))

    ;; and independently, both must match the S-mem-gate's known-correct shape (a second witness —
    ;; not a substitute for the differential above).
    (:wat::test::assert-eq (:wat::core::count (:wat::query::Page/rows (:probe::RunResult/page1 mem-result))) 2)
    (:wat::test::assert-eq (:wat::query::Page/next-cursor (:probe::RunResult/page3 mem-result)) :wat::core::None)
    (:wat::test::assert-eq (:wat::core::count (:wat::query::IndexPage/rows (:probe::RunResult/ipage sqlite-result))) 2)))

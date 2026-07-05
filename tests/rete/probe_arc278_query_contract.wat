;; Co-located fixture for probe_arc278_query_contract.rs — arc 278 stone S0 acceptance gate.
;;
;; Proves the :wat::query contract (Store/ReadStore surfaces + StoredRow/ScanRequest/Page
;; records) is REAL: a tiny in-file struct satisfier extend-types :wat::query::ReadStore and
;; dispatches `scan` through it, round-tripping a StoredRow's pk and a Page's next-cursor.

(:wat::core::defstruct :test::query::MemStore [tag <- :wat::core::i64])

(:wat::core::extend-type :test::query::MemStore :wat::query::ReadStore
  (scan [self q]
    (:wat::core::Ok (:wat::query::Page (:wat::core::Vector :wat::query::Row) :wat::core::None)))
  (scan-index [self q]
    (:wat::core::Ok (:wat::query::IndexPage (:wat::core::Vector :wat::query::IndexRow) :wat::core::None))))

(:wat::test::deftest' :user::query_contract ()
  (:wat::core::let
    [row      (:wat::query::StoredRow "part1" "sort1" "{}"
                (:wat::core::HashMap :wat::core::String :wat::query::IndexKey))
     req      (:wat::query::ScanRequest "part1" "a" "z" 10 :wat::core::None)
     store    (:test::query::MemStore 1)
     page-res (:wat::query::ReadStore/scan store req)
     page     (:wat::core::Result/expect page-res "scan should succeed")]
    (:wat::test::assert-eq (:wat::query::StoredRow/pk row) "part1")
    (:wat::test::assert-eq (:wat::query::Page/next-cursor page) :wat::core::None)))

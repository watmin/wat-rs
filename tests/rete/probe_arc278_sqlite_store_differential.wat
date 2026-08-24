;; Co-located fixture for probe_arc278_sqlite_store_differential.rs — arc 278 stone S4 the
;; DIFFERENTIAL acceptance gate: the sqlite `:wat::query::Store` satisfier (`sqlite-store'`,
;; wat/query/sqlite-store.wat) held bit-for-bit against the S-mem mem-store' oracle
;; (wat/query/mem.wat, proven by probe_arc278_smem_roundtrip.wat). Both are `:satisfies
;; :wat::query::Store` services on the operation model — a dialed peer IS the Store,
;; intrinsically (arc 293 Path B); no wrapper struct, no extend-type.
;;
;; ONE `run-ops` fn drives the op sequence (mirroring the S-mem gate exactly: ensure-schema with
;; one GSI "by-v" -> put 5 rows on pk "u#1" [2 projecting the GSI] -> keyset-paginate scan 2/2/1 ->
;; scan-index) through the SURFACE `:wat::query::Store` type, so the SAME code exercises BOTH
;; backends. Each op's outcome enum is matched to `:Success` and its rows+cursor boxed into a
;; `Page`/`IndexPage` (the kept vocabulary shape) so the differential compares like-for-like. The
;; gate dials a mem-store' peer AND a sqlite-store' peer (":memory:"), runs `run-ops` on EACH, and
;; asserts the returned Pages are EQUAL — same ops -> same Pages, bit-for-bit.

(:wat::core::defrecord :probe::RunResult
  [page1 <- :wat::query::Page
   page2 <- :wat::query::Page
   page3 <- :wat::query::Page
   ipage <- :wat::query::IndexPage])

(:wat::core::defn :probe::expect-scan
  [resp <- (:wat::kernel::RecvOutcome :- [:wat::query::Store::ScanResponse])] -> :wat::query::Page
  (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
    ((:wat::query::Store::ScanResponse::Success rows cursor) (:wat::query::Page :rows rows :next-cursor cursor))
    (_ (:wat::kernel::assertion-failed! "scan failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::expect-scan-index
  [resp <- (:wat::kernel::RecvOutcome :- [:wat::query::Store::ScanIndexResponse])] -> :wat::query::IndexPage
  (:wat::core::match resp ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
    ((:wat::query::Store::ScanIndexResponse::Success rows cursor) (:wat::query::IndexPage :rows rows :next-cursor cursor))
    (_ (:wat::kernel::assertion-failed! "scan-index failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::run-ops [store <- :wat::query::Store] -> :probe::RunResult
  (:wat::core::let
    [empty-ik (:wat::core::HashMap :wat::core::String :wat::query::IndexKey)
     ik-a     (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey :ipk "u#1" :isk "v1"))
     ik-c     (:wat::core::HashMap :wat::core::String :wat::query::IndexKey "by-v" (:wat::query::IndexKey :ipk "u#1" :isk "v2"))
     rows     (:wat::core::Vector :wat::query::StoredRow
                (:wat::query::StoredRow :pk "u#1" :sk "a" :data "{:v 1}" :index-keys ik-a)
                (:wat::query::StoredRow :pk "u#1" :sk "b" :data "{:v 2}" :index-keys empty-ik)
                (:wat::query::StoredRow :pk "u#1" :sk "c" :data "{:v 3}" :index-keys ik-c)
                (:wat::query::StoredRow :pk "u#1" :sk "d" :data "{:v 4}" :index-keys empty-ik)
                (:wat::query::StoredRow :pk "u#1" :sk "e" :data "{:v 5}" :index-keys empty-ik))
     _es      (:wat::core::match
                (:wat::query::Store/ensure-schema store
                  (:wat::query::Store::EnsureSchemaRequest
                    :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
                    :indexes (:wat::core::Vector :wat::query::IndexSchema (:wat::query::IndexSchema :name "by-v" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk")))) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
                
                ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
                (_ (:wat::kernel::assertion-failed! "ensure-schema failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     _p       (:wat::core::match (:wat::query::Store/put store (:wat::query::Store::PutRequest rows)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv
                
                ((:wat::query::Store::PutResponse::Success) nil)
                (_ (:wat::kernel::assertion-failed! "put failed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))

     page1    (:probe::expect-scan
                (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor :wat::core::None)))
     page2    (:probe::expect-scan
                (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor (:wat::core::Some "b"))))
     page3    (:probe::expect-scan
                (:wat::query::Store/scan store
                  (:wat::query::Store::ScanRequest :pk "u#1" :sk-lo "a" :sk-hi "z" :limit 2 :cursor (:wat::core::Some "d"))))
     ipage    (:probe::expect-scan-index
                (:wat::query::Store/scan-index store
                  (:wat::query::Store::ScanIndexRequest
                    :index "by-v" :ipk "u#1" :isk-lo "v1" :isk-hi "v2" :limit 10 :cursor :wat::core::None)))]
    (:probe::RunResult :page1 page1 :page2 page2 :page3 page3 :ipage ipage)))

(:wat::test::deftest :user::run-ops-on-mem-store
  (:wat::core::let
    [h         (:wat::query::mem-store/start :locus (:wat::spawn::thread)
                 :record (:wat::query::mem-store::Record (:wat::core::PersistentVector)))
     mem-store (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     result    (:probe::run-ops mem-store)]
    (:wat::test::assert-eq (:wat::core::count (:wat::query::Page/rows (:probe::RunResult/page1 result))) 2)
    (:wat::test::assert-eq (:wat::query::Page/next-cursor (:probe::RunResult/page3 result)) :wat::core::None)
    (:wat::test::assert-eq (:wat::core::count (:wat::query::IndexPage/rows (:probe::RunResult/ipage result))) 2)))

(:wat::test::deftest :user::sqlite_store_differential 
  (:wat::core::let
    [h            (:wat::query::mem-store/start :locus (:wat::spawn::thread)
                    :record (:wat::query::mem-store::Record (:wat::core::PersistentVector)))
     mem-store    (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))

     ;; sqlite-store' is the sibling `:satisfies :wat::query::Store` service — start INLINE (scope
     ;; law), connect'; the dialed peer IS the Store. The durable Record carries the path
     ;; (":memory:") + the declared GSI-name set ("by-v").
     sh           (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
                    :record (:wat::query::sqlite-store::Record
                              :path        ":memory:"
                              :index-names (:wat::core::Vector :wat::core::String "by-v")))
     sqlite-store (:wat::core::match (:wat::kernel::connect (:wat::query::sqlite-store::Handle/addr sh)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))

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

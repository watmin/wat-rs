;; Co-located fixture for probe_arc301_delete_differential.rs — arc 301 stone 2b.
;;
;; THE DIFFERENTIAL: the SAME delete sequence over mem-store' (oracle) and
;; sqlite-store' (:memory:). Shape copied from
;; tests/services/probe_arc278_journal_backend_differential.wat: a helper
;; parameterized on Address', run against both backends, return the shared
;; summary IFF they match, else a DIFFERENTIAL-MISMATCH sentinel carrying both
;; payloads (so a disagreement is evidence, not a bool).
;;
;; ★ A GSI is mandatory. Stone 2's STOP-2 argued a (pk, sk) Key is sufficient
;; because clear-index-projections deletes index_<name> rows by those columns.
;; An empty :index-names returns Ok immediately (sqlite-store.wat:155) and
;; proves nothing. This fixture declares one index, puts rows that PROJECT into
;; it (including the deleted row), and drives scan-index AFTER the delete on
;; both backends. An orphaned projection is a real bug, not a fixture problem.
;;
;; Also closes stone 2 finding 2: delete the same key twice (duplicate ack).
;; Both must return :Success; the second is a no-op.

(:wat::core::defn :user::connect-store
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :user::three-rows [] -> (:wat::core::Vector :- [:wat::query::StoredRow])
  (:wat::core::let
    [ik-a (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
            "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v1"))
     ik-b (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
            "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v2"))
     ik-c (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
            "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v3"))]
    (:wat::core::Vector :- [:wat::query::StoredRow]
      (:wat::query::StoredRow :pk "q#1" :sk "a" :data "{:v 1}" :index-keys ik-a)
      (:wat::query::StoredRow :pk "q#1" :sk "b" :data "{:v 2}" :index-keys ik-b)
      (:wat::query::StoredRow :pk "q#1" :sk "c" :data "{:v 3}" :index-keys ik-c))))

(:wat::core::defn :user::ensure-schema-with-gsi [store <- :wat::query::Store] -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema store
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes (:wat::core::Vector :- [:wat::query::IndexSchema]
                   (:wat::query::IndexSchema :name "by-v" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "ensure-schema did not succeed" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "ensure-schema: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::put-rows [store <- :wat::query::Store
                                  rows  <- (:wat::core::Vector :- [:wat::query::StoredRow])] -> :wat::core::nil
  (:wat::core::match (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::PutResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "put did not succeed" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "put: recv failed" :wat::core::None :wat::core::None))))

;; Encode the delete outcome in the summary — do not assertion-failed! on a
;; non-Success; a backend disagreement about the arm is the measurement.
(:wat::core::defn :user::delete-b-outcome [store <- :wat::query::Store] -> :wat::core::String
  (:wat::core::match
    (:wat::query::Store/delete store
      (:wat::query::Store::DeleteRequest
        (:wat::core::Vector :- [:wat::query::Key] (:wat::query::Key :pk "q#1" :sk "b"))))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::DeleteResponse::Success) "Success")
        (_ "NotSuccess")))
    (_ "RecvFailed")))

(:wat::core::defn :user::join-sks [rows <- (:wat::core::Vector :- [:wat::query::Row])] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::mapv
      (:wat::core::fn [r <- :wat::query::Row] -> :wat::core::String (:wat::query::Row/sk r))
      rows)))

(:wat::core::defn :user::join-isks [rows <- (:wat::core::Vector :- [:wat::query::IndexRow])] -> :wat::core::String
  (:wat::string::join ","
    (:wat::core::mapv
      (:wat::core::fn [r <- :wat::query::IndexRow] -> :wat::core::String (:wat::query::IndexRow/isk r))
      rows)))

;; Render one scan page as "N:sk,sk,..." or "FAIL".
(:wat::core::defn :user::render-scan [store <- :wat::query::Store] -> :wat::core::String
  (:wat::core::match
    (:wat::query::Store/scan store
      (:wat::query::Store::ScanRequest :pk "q#1" :sk-lo "" :sk-hi "z" :limit 100 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::ScanResponse::Success rows _c)
          (:wat::string::interpolate "{n}:{sks}"
            :n (:wat::i64::to-string (:wat::core::count rows))
            :sks (:user::join-sks rows)))
        (_ "FAIL")))
    (_ "FAIL")))

;; ★ scan-index AFTER the delete. isk range v1..v3 includes v2 so an orphaned
;; projection of the deleted row is visible, not excluded by the query.
(:wat::core::defn :user::render-gsi [store <- :wat::query::Store] -> :wat::core::String
  (:wat::core::match
    (:wat::query::Store/scan-index store
      (:wat::query::Store::ScanIndexRequest
        :index "by-v" :ipk "q#1" :isk-lo "v0" :isk-hi "vz" :limit 10 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::ScanIndexResponse::Success rows _c)
          (:wat::string::interpolate "{n}:{isks}"
            :n (:wat::i64::to-string (:wat::core::count rows))
            :isks (:user::join-isks rows)))
        (_ "FAIL")))
    (_ "FAIL")))


;; ── THE QUESTION: is `put` of an EXISTING (pk,sk) a REPLACE or an APPEND? ───────
;; Stone 3's queue design would make a message invisible by RE-PUTTING its row with a
;; new visible-at index-key. sqlite's put-one-row is DELETE->clear->INSERT->reindex, i.e.
;; a replace that MOVES the projection. mem's put is a bare `conj`. If they differ, the
;; design is unbuildable as written.
(:wat::core::defn :user::reput-roundtrip
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::core::String
  (:wat::core::let
    [store (:user::connect-store addr)
     _es   (:user::ensure-schema-with-gsi store)
     ik1   (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v1"))
     ik9   (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v9"))
     _p1   (:user::put-rows store (:wat::core::Vector :- [:wat::query::StoredRow]
              (:wat::query::StoredRow :pk "q#1" :sk "a" :data "{:v 1}" :index-keys ik1)))
     _p2   (:user::put-rows store (:wat::core::Vector :- [:wat::query::StoredRow]
              (:wat::query::StoredRow :pk "q#1" :sk "a" :data "{:v 1}" :index-keys ik9)))
     base  (:user::render-scan store)
     gsi   (:user::render-gsi store)]
    (:wat::string::interpolate "base={b};gsi={g}" :b base :g gsi)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     ssh   (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::sqlite-store::Record :path ":memory:"
                       :index-names (:wat::core::Vector :- [:wat::core::String] "by-v")))
     mem   (:user::reput-roundtrip (:wat::query::mem-store::Handle/addr msh))
     sql   (:user::reput-roundtrip (:wat::query::sqlite-store::Handle/addr ssh))]
    (:wat::string::interpolate "MEM[{m}]  SQLITE[{s}]" :m mem :s sql)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::compute)))

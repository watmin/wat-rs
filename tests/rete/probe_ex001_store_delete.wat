;; PROBE-store-has-no-delete.wat — excursus 001 stone 2, the DISCONFIRMING probe.
;;
;; RED AT HEAD, on EXACTLY one thing: `:wat::query::Store/delete` does not exist.
;; GREEN AFTER THE STONE, with no other edit to this file.
;;
;; Everything around the gap is deliberately proven-clean: `mem-store/start`, `connect`,
;; `ensure-schema`, `put` and `scan` are all copied from the green
;; tests/rete/probe_arc278_smem_roundtrip.wat. If this file fails on anything OTHER than
;; the missing `delete`, the failure is in the copy, not in the claim — go read that file.
;;
;; THE CLAIM: `:wat::query::Store`'s `:features` block (wat/query.wat:551-569) is exactly four
;; verbs — ensure-schema, put, scan, scan-index. Append and read. So SQS's `ack`, which must
;; REMOVE a message, is not expressible on today's Store.
;;
;; WHAT GREEN LOOKS LIKE: put 3 rows under one pk, delete the middle one by (pk,sk), scan the
;; full range, get 2 back — and the two survivors are "a" and "c", so the delete removed
;; exactly the row named and not a neighbour.
;;
;; NOTE the store is a REAL defservice actor, so `delete` must return a per-op outcome enum
;; (`Store::DeleteResponse`, `:Success` first) like every other op, and mutate durable state
;; visible to a LATER, SEPARATE `scan` on the same store. mem-store never errors, so every
;; response here is matched to its `:Success` arm.

(:wat::core::defn :test::three-rows [] -> (:wat::core::Vector :- [:wat::query::StoredRow])
  (:wat::core::let
    [empty-ik (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey])]
    (:wat::core::Vector :- [:wat::query::StoredRow]
      (:wat::query::StoredRow :pk "q#1" :sk "a" :data "{:v 1}" :index-keys empty-ik)
      (:wat::query::StoredRow :pk "q#1" :sk "b" :data "{:v 2}" :index-keys empty-ik)
      (:wat::query::StoredRow :pk "q#1" :sk "c" :data "{:v 3}" :index-keys empty-ik))))

(:wat::core::defn :test::ensure-schema [store <- :wat::query::Store] -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema store
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes (:wat::core::Vector :- [:wat::query::IndexSchema])))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "ensure-schema did not succeed" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "ensure-schema: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::put [store <- :wat::query::Store
                              rows  <- (:wat::core::Vector :- [:wat::query::StoredRow])] -> :wat::core::nil
  (:wat::core::match (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::PutResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "put did not succeed" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "put: recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::scan-count [store <- :wat::query::Store] -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/scan store
      (:wat::query::Store::ScanRequest :pk "q#1" :sk-lo "" :sk-hi "z" :limit 100 :cursor :wat::core::None))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::ScanResponse::Success rows _c) (:wat::core::count rows))
        (_ -1)))
    (_ -1)))

;; ★ THE GAP, in one call. Nothing else in this file is in question.
(:wat::core::defn :test::delete-b [store <- :wat::query::Store] -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/delete store
      (:wat::query::Store::DeleteRequest
        (:wat::core::Vector :- [:wat::query::Key] (:wat::query::Key :pk "q#1" :sk "b"))))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::query::Store::DeleteResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "delete did not succeed" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "delete: recv failed" :wat::core::None :wat::core::None))))

;; start+connect stays INLINE — spawn is lexical, and a helper that returns the peer leaves the
;; service thread dead (the law recorded in probe_arc278_smem_roundtrip.wat's header).
(:wat::test::deftest :user::delete-removes-exactly-the-named-row
  (:wat::core::let
    [h (:wat::query::mem-store/start :locus (:wat::spawn::thread)
         :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store (:wat::core::match (:wat::kernel::connect (:wat::query::mem-store::Handle/addr h))
             ((:wat::kernel::ConnectOutcome::Connected p) p)
             ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
             ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _e (:test::ensure-schema store)
     _p (:test::put store (:test::three-rows))
     before (:test::scan-count store)
     _d (:test::delete-b store)
     after  (:test::scan-count store)]
    (:wat::core::let
      [_1 (:wat::test::assert-eq before 3)]
      (:wat::test::assert-eq after 2))))

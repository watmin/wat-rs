;; Co-located fixture for probe_ex001_reput_differential.rs — excursus 001 stone 2c.
;;
;; Promoted from docs/excursus/2026/08/001-sns-sqs/PROBE-reput-divergence.wat with the
;; standalone `:user::main` dropped (the .rs drives `:user::compute`).
;;
;; THE QUESTION: is `put` of an EXISTING (pk,sk) a REPLACE or an APPEND?
;; At HEAD-before-2c, mem appended (`base=2:a,a;gsi=2:v1,v9`) and sqlite replaced
;; (`base=1:a;gsi=1:v9`). PutItem replaces; after the fix both sides must read
;; `base=1:a;gsi=1:v9`. Compute returns that shared summary IFF they match, else
;; a DIFFERENTIAL-MISMATCH sentinel carrying both payloads.

(:wat::core::defn :user::connect-store
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

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

;; isk range covers v1 and v9 so a leftover old projection is visible.
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
    (:wat::core::if (:wat::core::= mem sql)
      mem
      (:wat::string::interpolate "DIFFERENTIAL-MISMATCH mem={m} sqlite={s}" :m mem :s sql))))

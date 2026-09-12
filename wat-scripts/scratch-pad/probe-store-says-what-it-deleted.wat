;; Row 5–7 of the-store-says-what-it-deleted. A green floor cannot give this:
;; every corpus arm binds `_`. This probe READS `deleted`.
;;
;;   3 keys, 2 exist → deleted = 2  (mem AND sqlite)
;;   same on sqlite WITH a GSI      → still 2, not 2+|indexes|
;;   1 missing key                  → deleted = 0, still Success
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-store-says-what-it-deleted.wat

(:wat::core::defn :p::connect
  [addr <- (:wat::kernel::Address :- [:wat::query::Store::Op :wat::query::Store::Reply])]
  -> :wat::query::Store
  (:wat::core::match (:wat::kernel::connect addr)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :p::ensure
  [store <- :wat::query::Store
   indexes <- (:wat::core::Vector :- [:wat::query::IndexSchema])]
  -> :wat::core::nil
  (:wat::core::match
    (:wat::query::Store/ensure-schema store
      (:wat::query::Store::EnsureSchemaRequest
        :table   (:wat::query::TableSchema :pk "pk" :sk "sk")
        :indexes indexes))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::EnsureSchemaResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "ensure-schema not Success" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "ensure-schema recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :p::two-rows [with-gsi <- :wat::core::bool]
  -> (:wat::core::Vector :- [:wat::query::StoredRow])
  (:wat::core::let
    [ik (:wat::core::if with-gsi
           (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]
             "by-v" (:wat::query::IndexKey :ipk "q#1" :isk "v1"))
           (:wat::core::HashMap :- [:wat::core::String :wat::query::IndexKey]))]
    (:wat::core::Vector :- [:wat::query::StoredRow]
      (:wat::query::StoredRow :pk "q#1" :sk "a" :data "{:v 1}" :index-keys ik)
      (:wat::query::StoredRow :pk "q#1" :sk "b" :data "{:v 2}" :index-keys ik))))

(:wat::core::defn :p::put
  [store <- :wat::query::Store  rows <- (:wat::core::Vector :- [:wat::query::StoredRow])]
  -> :wat::core::nil
  (:wat::core::match (:wat::query::Store/put store (:wat::query::Store::PutRequest rows))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::PutResponse::Success) nil)
        (_ (:wat::kernel::assertion-failed! "put not Success" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "put recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :p::deleted-of
  [store <- :wat::query::Store  keys <- (:wat::core::Vector :- [:wat::query::Key])]
  -> :wat::core::i64
  (:wat::core::match
    (:wat::query::Store/delete store (:wat::query::Store::DeleteRequest keys))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:wat::query::Store::DeleteResponse::Success n) n)
        (_ (:wat::kernel::assertion-failed! "delete not Success" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "delete recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :p::three-keys [] -> (:wat::core::Vector :- [:wat::query::Key])
  (:wat::core::Vector :- [:wat::query::Key]
    (:wat::query::Key :pk "q#1" :sk "a")
    (:wat::query::Key :pk "q#1" :sk "b")
    (:wat::query::Key :pk "q#1" :sk "missing")))

(:wat::core::defn :p::one-missing [] -> (:wat::core::Vector :- [:wat::query::Key])
  (:wat::core::Vector :- [:wat::query::Key]
    (:wat::query::Key :pk "q#1" :sk "missing")))

(:wat::core::defn :p::run-mem [label <- :wat::core::String  keys <- (:wat::core::Vector :- [:wat::query::Key]) put? <- :wat::core::bool]
  -> :wat::core::nil
  (:wat::core::let
    [h (:wat::query::mem-store/start :locus (:wat::spawn::thread)
         :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     store (:p::connect (:wat::query::mem-store::Handle/addr h))
     _ (:p::ensure store (:wat::core::Vector :- [:wat::query::IndexSchema]))
     _ (:wat::core::if put? (:p::put store (:p::two-rows false)) nil)
     n (:p::deleted-of store keys)
     _ (:wat::kernel::println (:wat::string::interpolate "{l}={n}" :l label :n n))
     _ (:wat::service::stop-faced (:wat::query::mem-store/stop h))]
    nil))

(:wat::core::defn :p::run-sqlite
  [label <- :wat::core::String
   keys <- (:wat::core::Vector :- [:wat::query::Key])
   put? <- :wat::core::bool
   gsi? <- :wat::core::bool]
  -> :wat::core::nil
  (:wat::core::let
    [indexes (:wat::core::if gsi?
                (:wat::core::Vector :- [:wat::query::IndexSchema]
                  (:wat::query::IndexSchema :name "by-v" :pk "pk" :sk "sk" :ipk "ipk" :isk "isk"))
                (:wat::core::Vector :- [:wat::query::IndexSchema]))
     names (:wat::core::if gsi?
              (:wat::core::Vector :- [:wat::core::String] "by-v")
              (:wat::core::Vector :- [:wat::core::String]))
     h (:wat::query::sqlite-store/start :locus (:wat::spawn::thread)
         :record (:wat::query::sqlite-store::Record :path ":memory:" :index-names names))
     store (:p::connect (:wat::query::sqlite-store::Handle/addr h))
     _ (:p::ensure store indexes)
     _ (:wat::core::if put? (:p::put store (:p::two-rows gsi?)) nil)
     n (:p::deleted-of store keys)
     _ (:wat::kernel::println (:wat::string::interpolate "{l}={n}" :l label :n n))
     _ (:wat::service::stop-faced (:wat::query::sqlite-store/stop h))]
    nil))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:p::run-mem "mem-partial" (:p::three-keys) true)
    (:p::run-sqlite "sqlite-partial" (:p::three-keys) true false)
    (:p::run-sqlite "sqlite-gsi" (:p::three-keys) true true)
    (:p::run-mem "mem-missing" (:p::one-missing) false)
    (:p::run-sqlite "sqlite-missing" (:p::one-missing) false false)
    nil))

;; BRIEF-STONE-the-dormant-minter.md row 3 — a parametric `defservice` round-trips.
;; Drives `:wat::cache::lru-svc :- [K V]` (wat/cache.wat) end to end: start as a
;; thread, connect, put an entry, get it back. K=String, V=i64, pinned explicitly
;; via ann-form (neither :durable nor :locus carries K/V for /start to infer from).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::core::ann-form
         (:wat::cache::lru-svc/start :locus (:wat::spawn::thread)
           :record (:wat::cache::lru-svc::Record :capacity 4))
         (:wat::cache::lru-svc::Handle :- [:wat::core::String :wat::core::i64]))
     c (:wat::core::match (:wat::kernel::connect (:wat::cache::lru-svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p) p)
         ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
         ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _put (:wat::core::match
            (:wat::cache::lru-svc/put c
              (:wat::cache::Cache::PutRequest :entries [(:wat::cache::Entry :key "k1" :value 42)]))
            ((:wat::kernel::RecvOutcome::Message _resp) nil)
            ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "put: stopped" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "put: closed" :wat::core::None :wat::core::None)))
     r (:wat::cache::lru-svc/get c (:wat::cache::Cache::GetRequest :probes ["k1"]))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat::cache::Cache::GetResponse::Ok results)
            (:wat::core::match (:wat::core::first results)
              ((:wat::cache::Cache::GetResult::Hit v) (:wat::kernel::println (:wat::string::interpolate "ROW3={v}" :v (:wat::core::i64::to-string v))))
              ((:wat::cache::Cache::GetResult::Miss) (:wat::kernel::assertion-failed! "ROW3: unexpected Miss" :wat::core::None :wat::core::None))))
          ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap) (:wat::kernel::assertion-failed! "ROW3: RequestTooLarge" :wat::core::None :wat::core::None))
          ((:wat::cache::Cache::GetResponse::RequestMalformed p e g) (:wat::kernel::assertion-failed! "ROW3: RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "get: stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "get: closed" :wat::core::None :wat::core::None)))))

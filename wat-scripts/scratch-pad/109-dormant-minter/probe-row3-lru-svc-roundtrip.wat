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
         [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
         [:wat::kernel::ConnectOutcome.Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
         [:wat::kernel::ConnectOutcome.Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))]
         [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     _put (:wat::core::match
            (:wat::cache::lru-svc/put c
              (:wat::cache::Cache::PutRequest :entries [(:wat::cache::Entry :key "k1" :value 42)]))
            [:wat::kernel::RecvOutcome.Message {:msg _resp} nil]
            [:wat::kernel::RecvOutcome.Lost {:cause cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
            [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "put: stopped")]
            [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "put: closed")])
     r (:wat::cache::lru-svc/get c (:wat::cache::Cache::GetRequest :probes ["k1"]))]
    (:wat::core::match r
      [:wat::kernel::RecvOutcome.Message {:msg resp}
        (:wat::core::match resp
          [:wat::cache::Cache::GetResponse.Ok {:results results}
            (:wat::core::match (:wat::core::first results)
              [:wat::cache::Cache::GetResult.Hit {:value v} (:wat::kernel::println (:wat::string::interpolate "ROW3={v}" :v (:wat::i64::to-string v)))]
              [:wat::cache::Cache::GetResult.Miss {} (:wat::kernel::assertion-failed! :message "ROW3: unexpected Miss")])]
          [:wat::cache::Cache::GetResponse.RequestTooLarge {:bytes bytes :cap cap} (:wat::kernel::assertion-failed! :message "ROW3: RequestTooLarge")]
          [:wat::cache::Cache::GetResponse.RequestMalformed {:path p :expected e :got g} (:wat::kernel::assertion-failed! :message "ROW3: RequestMalformed")])]
      [:wat::kernel::RecvOutcome.Lost {:cause cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
      [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "get: stopped")]
      [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "get: closed")])))

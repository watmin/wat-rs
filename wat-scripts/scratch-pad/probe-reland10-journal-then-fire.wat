;; RELAND 10: query-logs in MAIN, then the generated fire-rules foldl on those
;; logs — still outside the sift service.
(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

(:wat::core::defn :user::hot-q [] -> :wat::rete::Query
  (:wat::rete::make-query "usr::Hot" (:wat::core::quote [])
    (:wat::core::quote [(?fact <- :usr::Hot)])))

(:wat::core::defn :user::warn-q [] -> :wat::rete::Query
  (:wat::rete::make-query "usr::Warn" (:wat::core::quote [])
    (:wat::core::quote [(?fact <- :usr::Warn)])))

(:wat::core::defn :user::run-one [session <- :wat::rete::Session encoded <- :wat::core::String]
  -> (:wat::core::PersistentVector :- [:wat::core::Value])
  (:wat::core::let
    [fired (:wat::rete::fire-rules
             (:wat::rete::insert session (:wat::edn::read encoded)))]
    (:wat::core::concat
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::Value
            (:wat::core::Option/expect (:wat::map::get p "?fact") "sift-rules: ?fact"))
          (:wat::rete::query fired (:user::hot-q))))
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::map
          (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::Value
            (:wat::core::Option/expect (:wat::map::get p "?fact") "sift-rules: ?fact"))
          (:wat::rete::query fired (:user::warn-q)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::core::PersistentVector
             (:wat::rete::make-rule "usr::hot-rule"
               (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))])
               (:wat::core::quote [(:usr::Hot :c ?c)]))
             (:wat::rete::make-rule "usr::warn-rule"
               (:wat::core::quote [(:usr::Temp (?c <- :c) (:wat::rete::i64::> ?c 50))])
               (:wat::core::quote [(:usr::Warn :c ?c)])))
     queries (:wat::core::PersistentVector (:user::hot-q) (:user::warn-q))
     session (:wat::rete::compile-all rules queries)
     msh   (:wat::query::mem-store/start :locus (:wat::spawn::thread)
             :record (:wat::query::mem-store::Record :rows (:wat::core::PersistentVector)))
     maddr (:wat::query::mem-store::Handle/addr msh)
     jh    (:wat::telemetry::journal/start :locus (:wat::spawn::thread)
             :record (:wat::telemetry::journal::Record) :store-addr maddr)
     jaddr (:wat::telemetry::journal::Handle/addr jh)
     journal (:wat::core::match (:wat::kernel::connect jaddr)
               [:wat::kernel::ConnectOutcome.Connected {:peer p} p]
               [_ (:wat::kernel::assertion-failed! :message "connect journal")])
     tags  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String])
     idxs  (:wat::core::range 0 240)
     logs  (:wat::core::into (:wat::core::Vector :- [:wat::telemetry::Log])
             (:wat::core::map
               (:wat::core::fn [i <- :wat::core::i64] -> :wat::telemetry::Log
                 (:wat::core::let
                   [hot? (:wat::i64::< i 30)
                    c    (:wat::core::if hot? 60 10)
                    msg  (:wat::edn::write (:usr::Temp :c c))]
                   (:wat::telemetry::Log :namespace "sift-rules-ns" :uuid (:wat::uuid::nil) :tags tags
                     :time-ns (:wat::i64::+ i 1) :emitted-from (:wat::kernel::call-site)
                     :level :wat::telemetry::Level.Info :message msg)))
               idxs))
     _wr   (:wat::telemetry::Journal/write-logs journal
             (:wat::telemetry::Journal::WriteLogsRequest logs))
     qresp (:wat::telemetry::Journal/query-logs journal
             (:wat::telemetry::Journal::QueryLogsRequest
               :namespace "sift-rules-ns" :time-lo 0 :time-hi 100000 :limit 300
               :cursor :wat::core::Option.None))]
    (:wat::kernel::println "QUERY-LOGS")
    (:wat::core::match qresp
      [:wat::kernel::RecvOutcome.Message {:msg sresp}
        (:wat::core::match sresp
          [:wat::telemetry::Journal::QueryLogsResponse.Success {:logs got :cursor _c}
            (:wat::core::do
              (:wat::kernel::println "LOGS")
              (:wat::kernel::pprintln (:wat::core::count got))
              (:wat::core::let
                [items (:wat::core::foldl
                         (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Value])
                                          log <- :wat::telemetry::Log]
                           -> (:wat::core::PersistentVector :- [:wat::core::Value])
                           (:wat::core::concat acc
                             (:user::run-one session (:wat::telemetry::Log/message log))))
                         (:wat::core::PersistentVector)
                         got)]
                (:wat::core::do
                  (:wat::kernel::println "AFTER-FIRE")
                  (:wat::kernel::pprintln (:wat::core::count items)))))]
          [_ (:wat::kernel::println "QUERY-NOT-SUCCESS")])]
      [:wat::kernel::RecvOutcome.Lost {:cause c}
        (:wat::core::do
          (:wat::kernel::println "QUERY-LOST")
          (:wat::kernel::pprintln (:wat::kernel::LociDiedError/message c)))]
      [_ (:wat::kernel::println "QUERY-OTHER")])
    nil))

;; RELAND 10: drive fire-rules DIRECTLY on the same rules and facts the
;; generated sift-rules body uses — outside the service, so a raise is the
;; service's own error instead of the client's Lost/"disconnected".
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
     queries (:wat::core::PersistentVector
               (:user::hot-q)
               (:user::warn-q))
     session (:wat::rete::compile-all rules queries)
     one (:wat::edn::write (:usr::Temp :c 60))
     items1 (:user::run-one session one)]
    (:wat::kernel::println "ONE-FACT")
    (:wat::kernel::pprintln (:wat::core::count items1))
    (:wat::core::let
      [idxs (:wat::core::range 0 240)
       items240 (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Value])
                                   i <- :wat::core::i64]
                    -> (:wat::core::PersistentVector :- [:wat::core::Value])
                    (:wat::core::let
                      [hot? (:wat::i64::< i 30)
                       c    (:wat::core::if hot? 60 10)
                       enc  (:wat::edn::write (:usr::Temp :c c))]
                      (:wat::core::concat acc (:user::run-one session enc))))
                  (:wat::core::PersistentVector)
                  idxs)]
      (:wat::kernel::println "TWO-FORTY")
      (:wat::kernel::pprintln (:wat::core::count items240))
      nil)))

;; RELAND 9 STOP-2: isolate fire-rules/insert the generated sift body takes.
(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

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
               (:wat::rete::make-query "usr::Hot" (:wat::core::quote []) (:wat::core::quote [(?fact <- :usr::Hot)]))
               (:wat::rete::make-query "usr::Warn" (:wat::core::quote []) (:wat::core::quote [(?fact <- :usr::Warn)])))
     session (:wat::rete::compile-all rules queries)
     encoded (:wat::edn::write (:usr::Temp :c 60))
     fact (:wat::edn::read encoded)
     fired (:wat::rete::fire-rules (:wat::rete::insert session fact))]
    (:wat::kernel::println "FIRED-OK")
    (:wat::kernel::pprintln fired)
    nil))

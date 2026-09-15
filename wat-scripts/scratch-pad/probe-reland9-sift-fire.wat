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
     session (:wat::core::match (:wat::rete::compile-all rules queries) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     encoded (:wat::edn::write (:usr::Temp :c 60))
     fact (:wat::edn::read encoded)
     fired (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert session fact) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))]
    (:wat::kernel::println "FIRED-OK")
    (:wat::kernel::pprintln fired)
    nil))

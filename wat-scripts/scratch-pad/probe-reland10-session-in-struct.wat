;; RELAND 10: does fire-rules die when the Session was stored in a defstruct
;; field (the service's ephemeral :template) rather than a let-local?
(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])
(:wat::core::defstruct :r10::Box [template <- :wat::rete::Session])

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
               (:wat::rete::make-query "usr::Hot" (:wat::core::quote [])
                 (:wat::core::quote [(?fact <- :usr::Hot)]))
               (:wat::rete::make-query "usr::Warn" (:wat::core::quote [])
                 (:wat::core::quote [(?fact <- :usr::Warn)])))
     session (:wat::rete::compile-all rules queries)
     boxed (:r10::Box :template session)
     pulled (:r10::Box/template boxed)
     fact (:wat::edn::read (:wat::edn::write (:usr::Temp :c 60)))]
    (:wat::kernel::println "STORED-AND-PULLED")
    (:wat::core::let
      [inserted (:wat::rete::insert pulled fact)]
      (:wat::kernel::println "INSERTED")
      (:wat::core::let
        [fired (:wat::rete::fire-rules inserted)]
        (:wat::kernel::println "FIRED")
        (:wat::kernel::pprintln (:wat::core::count (:wat::rete::Session/facts fired)))
        nil))))

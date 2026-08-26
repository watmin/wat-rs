;; :not of :and whose leaf is a DERIVED Bad. 7strat is bare :not of a fact.
;; rule_negates must raise Ok above Bad, not store "wat::rete::and".
;; Want Bad=1 Ok=1 (A(1) only), not Ok=2.

(:wat::core::defrecord :nad::A   [k <- :wat::core::i64])
(:wat::core::defrecord :nad::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :nad::Ok  [k <- :wat::core::i64])

(:wat::rete::defrule :nad::mark-bad
  :when [(:nad::A (?k <- :k))
         (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:nad::Bad :k ?k)])

(:wat::rete::defrule :nad::ok
  :when [(:nad::A (?k <- :k))
         (:wat::rete::not
           (:wat::rete::and
             (:nad::Bad (?k <- :k))))]
  :then [(:nad::Ok :k ?k)])

(:wat::rete::defquery :nad::q-Bad :params [] :when [(?f <- :nad::Bad)])
(:wat::rete::defquery :nad::q-Ok  :params [] :when [(?f <- :nad::Ok)])

(:wat::core::defn :nad::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s (:nad::A :k 1) (:nad::A :k 2)))

(:wat::core::defn :nad::counts [fired <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query fired (:nad::q-Bad)))
    (:wat::core::length (:wat::rete::query fired (:nad::q-Ok)))))

(:wat::core::defn :user::source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:nad::counts
    (:wat::rete::fire-rules
      (:nad::seed
        (:wat::rete::compile-all
          (:wat::core::PersistentVector (:nad::mark-bad) (:nad::ok))
          (:wat::core::PersistentVector (:nad::q-Bad) (:nad::q-Ok)))))))

(:wat::core::defn :user::spec-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:nad::counts
    (:wat::rete::fire-rules$oracle
      (:nad::seed
        (:wat::rete::compile-all
          (:wat::core::PersistentVector (:nad::mark-bad) (:nad::ok))
          (:wat::core::PersistentVector (:nad::q-Bad) (:nad::q-Ok)))))))

(:wat::core::defn :user::import-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0  (:wat::rete::compile-all
                          (:wat::core::PersistentVector (:nad::mark-bad) (:nad::ok))
                          (:wat::core::PersistentVector (:nad::q-Bad) (:nad::q-Ok)))
                    exp (:wat::rete::export s0)
                    s1  (:wat::rete::import exp)]
    (:nad::counts (:wat::rete::fire-rules (:nad::seed s1)))))

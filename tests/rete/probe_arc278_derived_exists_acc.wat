;; :exists and acc :from a type THIS FIRE derives (raised by :not of Bad).
;; Circumspicere: every prior exists/acc row used inserted Readings.
;; Gate: native fire + Export import. Want [Bad=1 Ok=1 Seen=1 Tally=1].

(:wat::core::defrecord :dea::A     [k <- :wat::core::i64])
(:wat::core::defrecord :dea::Seed  [id <- :wat::core::i64])
(:wat::core::defrecord :dea::Bad   [k <- :wat::core::i64])
(:wat::core::defrecord :dea::Ok    [k <- :wat::core::i64])
(:wat::core::defrecord :dea::Seen  [k <- :wat::core::i64])
(:wat::core::defrecord :dea::Tally [n <- :wat::core::i64])

(:wat::rete::defrule :dea::mark-bad
  :when [(:dea::A (?k <- :k))
         (:wat::rete::where (:wat::rete::i64::= ?k 2))]
  :then [(:dea::Bad :k ?k)])

(:wat::rete::defrule :dea::ok
  :when [(:dea::A (?k <- :k))
         (:wat::rete::not (:dea::Bad (?k <- :k)))]
  :then [(:dea::Ok :k ?k)])

(:wat::rete::defrule :dea::seen
  :when [(:dea::A (?k <- :k))
         (:wat::rete::exists (:dea::Ok (?k <- :k)))]
  :then [(:dea::Seen :k ?k)])

(:wat::rete::defrule :dea::tally
  :when [(:dea::Seed (?id <- :id))
         (?n <- (:wat::rete::acc::count) :from (:dea::Ok))
         (:wat::rete::where (:wat::rete::i64::= ?n 1))]
  :then [(:dea::Tally :n ?n)])

(:wat::rete::defquery :dea::q-Bad   :params [] :when [(?f <- :dea::Bad)])
(:wat::rete::defquery :dea::q-Ok    :params [] :when [(?f <- :dea::Ok)])
(:wat::rete::defquery :dea::q-Seen  :params [] :when [(?f <- :dea::Seen)])
(:wat::rete::defquery :dea::q-Tally :params [] :when [(?f <- :dea::Tally)])

(:wat::core::defn :dea::rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector (:dea::mark-bad) (:dea::ok) (:dea::seen) (:dea::tally)))

(:wat::core::defn :dea::queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector (:dea::q-Bad) (:dea::q-Ok) (:dea::q-Seen) (:dea::q-Tally)))

(:wat::core::defn :dea::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s
    (:dea::Seed :id 1)
    (:dea::A :k 1)
    (:dea::A :k 2)))

(:wat::core::defn :dea::counts [fired <- :wat::rete::Session]
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query fired (:dea::q-Bad)))
    (:wat::core::length (:wat::rete::query fired (:dea::q-Ok)))
    (:wat::core::length (:wat::rete::query fired (:dea::q-Seen)))
    (:wat::core::length (:wat::rete::query fired (:dea::q-Tally)))))

(:wat::core::defn :user::source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:dea::counts
    (:wat::rete::fire-rules
      (:dea::seed (:wat::rete::compile-all (:dea::rules) (:dea::queries))))))

(:wat::core::defn :user::import-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0  (:wat::rete::compile-all (:dea::rules) (:dea::queries))
                    exp (:wat::rete::export s0)
                    s1  (:wat::rete::import exp)]
    (:dea::counts (:wat::rete::fire-rules (:dea::seed s1)))))

(:wat::core::defn :user::spec-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:dea::counts
    (:wat::rete::fire-rules$oracle
      (:dea::seed (:wat::rete::compile-all (:dea::rules) (:dea::queries))))))

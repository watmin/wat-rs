;; #wat.rete/Export — compiled program from source. Native fire only.

(:wat::core::defrecord :exp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :exp::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :exp::q-Hit :params [] :when [(?fact <- :exp::Hit)])

(:wat::rete::defrule :exp::cool
  :when [(:exp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:exp::Hit ?c)])

(:wat::core::defn :exp::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:exp::Temp :c 10))
    (:exp::Temp :c 30)))

(:wat::core::defn :user::source-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s0)) (:exp::q-Hit)))))

(:wat::core::defn :user::import-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    s1 (:wat::rete::import exp)]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s1)) (:exp::q-Hit)))))

(:wat::core::defn :user::export-sizes [] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    sl (:wat::core::string::length (:wat::edn::write s0))
                    el (:wat::core::string::length (:wat::edn::write exp))
                    nc (:wat::core::length (:wat::rete::Export/classes exp))
                    nn (:wat::core::length (:wat::rete::Export/nodes exp))
                    ncond (:wat::core::length (:wat::rete::Export/conds exp))]
    (:wat::core::PersistentVector sl el nc nn ncond)))

;; Negation over a DERIVED fact. Source fire is stratified. Import must match.
(:wat::core::defrecord :sn::A   [k <- :wat::core::i64])
(:wat::core::defrecord :sn::Bad [k <- :wat::core::i64])
(:wat::core::defrecord :sn::Ok  [k <- :wat::core::i64])

(:wat::rete::defrule :sn::mark-bad
  :when [(:sn::A (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k 2))]
  :then [(:sn::Bad :k ?k)])

(:wat::rete::defrule :sn::ok
  :when [(:sn::A (?k <- :k))
         (:wat::rete::not (:sn::Bad (?k <- :k)))]
  :then [(:sn::Ok :k ?k)])

(:wat::rete::defquery :sn::q-Bad :params [] :when [(?fact <- :sn::Bad)])
(:wat::rete::defquery :sn::q-Ok  :params [] :when [(?fact <- :sn::Ok)])

(:wat::core::defn :sn::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:sn::A :k 1))
    (:sn::A :k 2)))

(:wat::core::defn :sn::counts [fired <- :wat::rete::Session] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::PersistentVector
    (:wat::core::length (:wat::rete::query fired (:sn::q-Bad)))
    (:wat::core::length (:wat::rete::query fired (:sn::q-Ok)))))

(:wat::core::defn :user::strat-source-counts [] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:sn::mark-bad) (:sn::ok))
                         (:wat::core::PersistentVector (:sn::q-Bad) (:sn::q-Ok)))]
    (:sn::counts (:wat::rete::fire-rules (:sn::seed s0)))))

(:wat::core::defn :user::strat-import-counts [] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:sn::mark-bad) (:sn::ok))
                         (:wat::core::PersistentVector (:sn::q-Bad) (:sn::q-Ok)))
                    exp (:wat::rete::export s0)
                    s1 (:wat::rete::import exp)]
    (:sn::counts (:wat::rete::fire-rules (:sn::seed s1)))))

(:wat::core::defn :user::edn-roundtrip-hits [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))
                    exp (:wat::rete::export s0)
                    txt (:wat::edn::write exp)
                    exp2 (:wat::edn::read txt)
                    s1 (:wat::rete::import exp2)]
    (:wat::core::length
      (:wat::rete::query (:wat::rete::fire-rules (:exp::seed s1)) (:exp::q-Hit)))))

;; The compiled program as an EDN string — source of tests/rete/hello.rete.edn.
(:wat::core::defn :user::export-edn [] -> :wat::core::String
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:exp::cool))
                         (:wat::core::PersistentVector (:exp::q-Hit)))]
    (:wat::edn::write (:wat::rete::export s0))))

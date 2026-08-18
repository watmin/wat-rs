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

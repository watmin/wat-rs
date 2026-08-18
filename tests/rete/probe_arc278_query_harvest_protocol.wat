;; Harvest is fire-time. Insert does not refresh query-memory.
;; fire → query=1 → insert more → query still 1 → fire → query=2.

(:wat::core::defrecord :qhp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :qhp::Hit  [c <- :wat::core::i64])

(:wat::rete::defrule :qhp::cool
  :when [(:qhp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:qhp::Hit ?c)])

(:wat::rete::defquery :qhp::q-Hit :params [] :when [(?f <- :qhp::Hit)])

(:wat::core::defn :user::protocol [] -> :wat::core::PersistentVector<wat::core::i64>
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:qhp::cool))
                         (:wat::core::PersistentVector (:qhp::q-Hit)))
                    s1 (:wat::rete::insert s0 (:qhp::Temp :c 10))
                    f1 (:wat::rete::fire-rules s1)
                    n1 (:wat::core::length (:wat::rete::query f1 (:qhp::q-Hit)))
                    s2 (:wat::rete::insert f1 (:qhp::Temp :c 15))
                    n2 (:wat::core::length (:wat::rete::query s2 (:qhp::q-Hit)))
                    f2 (:wat::rete::fire-rules s2)
                    n3 (:wat::core::length (:wat::rete::query f2 (:qhp::q-Hit)))]
    (:wat::core::PersistentVector n1 n2 n3)))

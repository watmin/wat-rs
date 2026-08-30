;; fn-headed :then produces Hit; another rule exists that derived Hit.
;; rule_produces must list Hit, not the fn name, so exists raises.

(:wat::core::defrecord :fhp::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :fhp::Hit  [c <- :wat::core::i64])
(:wat::core::defrecord :fhp::Seen [c <- :wat::core::i64])

(:wat::rete::core::defn :fhp::as-hit [c <- :wat::core::i64] -> :fhp::Hit
  (:fhp::Hit :c c))

(:wat::rete::defrule :fhp::cool
  :when [(:fhp::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::core::i64::< ?c 20))]
  :then [(:fhp::as-hit ?c)])

(:wat::rete::defrule :fhp::seen
  :when [(:fhp::Temp (?c <- :c))
         (:wat::rete::exists (:fhp::Hit (?c <- :c)))]
  :then [(:fhp::Seen :c ?c)])

(:wat::rete::defquery :fhp::q-Hit  :params [] :when [(?f <- :fhp::Hit)])
(:wat::rete::defquery :fhp::q-Seen :params [] :when [(?f <- :fhp::Seen)])

(:wat::core::defn :user::source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:fhp::cool) (:fhp::seen))
                         (:wat::core::PersistentVector (:fhp::q-Hit) (:fhp::q-Seen)))
                    fired (:wat::core::match (:wat::rete::fire-rules
                            (:wat::rete::insert s0 (:fhp::Temp :c 10))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:fhp::q-Hit)))
      (:wat::core::length (:wat::rete::query fired (:fhp::q-Seen))))))

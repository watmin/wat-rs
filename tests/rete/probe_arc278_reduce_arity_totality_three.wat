;; ADMIT fixture — the 3-arity reduce is total; identical in every other byte.
(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- (:wat::core::PersistentVector :- [:wat::core::i64])])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (?v <- :v))
   (:wat::rete::where
     (:wat::rete::core::i64::=
       (:wat::rete::core::reduce
         (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
           (:wat::rete::core::i64::+ acc x :undefined 0))
         0 ?v)
       3))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::rete::insert session (:probe::In :k "hit" :v (:wat::core::PersistentVector 1 2)))
     fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))

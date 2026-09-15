;; REFUSE fixture — the 2-arity reduce is PARTIAL (raises on empty) and the row is total: true.
(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- (:wat::core::PersistentVector :- [:wat::core::i64])])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (?v <- :v))
   (:wat::rete::where
     (:wat::rete::i64::=
       (:wat::rete::core::reduce
         (:wat::rete::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
           (:wat::rete::i64::+ acc x :undefined 0))
         ?v)
       3))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::rete::insert session (:probe::In :k "hit" :v (:wat::core::PersistentVector 1 2)))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))

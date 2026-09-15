;; ADMIT fixture — the 3-arity reduce is total; identical in every other byte.
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
         0 ?v)
       3))]
  :then
  [(:probe::Out :k ?k)])

(:wat::rete::defquery :probe::q :params [] :when [(?fact <- :probe::Out)])

(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q)))
     session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit" :v (:wat::core::PersistentVector 1 2))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (:wat::core::length (:wat::rete::query fired (:probe::q)))))

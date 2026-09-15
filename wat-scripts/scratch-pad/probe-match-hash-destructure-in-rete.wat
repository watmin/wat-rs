;; The same arm, inside a rete `where`. Subject is a bound HolonAST-free record field.
(:wat::core::defrecord :md::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :md::In  [k <- :wat::core::String  p <- :md::Point])
(:wat::core::defrecord :md::Out [k <- :wat::core::String])

(:wat::rete::defrule :md::rule
  :when
  [(:md::In (?k <- :k) (?p <- :p))
   (:wat::rete::where
     (:wat::rete::i64::=
       (:wat::rete::core::match ?p
         [{vx :x  vy :y} (:wat::rete::i64::+ vx vy :undefined 0)])
       42))]
  :then
  [(:md::Out :k ?k)])

(:wat::rete::defquery :md::q :params [] :when [(?fact <- :md::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :md)
         session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:md::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
         session (:wat::core::match (:wat::rete::insert session (:md::In :k "hit"  :p (:md::Point :x 40 :y 2))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         session (:wat::core::match (:wat::rete::insert session (:md::In :k "miss" :p (:md::Point :x 1 :y 1))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
        (:wat::rete::query fired (:md::q))))))

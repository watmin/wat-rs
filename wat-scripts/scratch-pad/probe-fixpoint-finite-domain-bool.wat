;; MEASUREMENT — a computed head whose field type has a domain of TWO.
;; `flag <- bool`, `:then` derives `(not ?flag)`. The fact domain is {F(true), F(false)}.
;; It CANNOT diverge: two facts is the whole universe. Does the verifier refuse it anyway?
(:wat::core::defrecord :fd::F [flag <- :wat::core::bool])

(:wat::rete::defrule :fd::flip
  :when  [(:fd::F (?b <- :flag))]
  :then  [(:fd::F :flag (:wat::rete::core::not ?b))])

(:wat::rete::defquery :fd::q :params [] :when [(?fact <- :fd::F)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :fd)
         session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fd::q)))
         session (:wat::core::match (:wat::rete::insert session (:fd::F :flag true)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
         fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
        (:wat::rete::query fired (:fd::q))))))

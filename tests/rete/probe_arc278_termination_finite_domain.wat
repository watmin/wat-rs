;; A COMPUTED head whose field type has TWO inhabitants. The fact population is {F(true),F(false)};
;; the fixpoint converges after two rounds and cannot do otherwise.
;;
;; Refused for the life of the check until 2026-08-29. The cyclicity test measures RANGE
;; RESTRICTION — a syntactic property, "the head's value came from the body" — and finiteness is a
;; TYPE property, so a domain of two was refused exactly as an unbounded i64 counter is. Measured
;; with the check disarmed: bool converges at 2, enum(3) at 2, guarded i64 at 501, and unguarded
;; i64 aborts the allocator.
(:wat::core::defrecord :ft::F [flag <- :wat::core::bool])

(:wat::rete::defrule :ft::flip
  :when  [(:ft::F (?b <- :flag))]
  :then  [(:ft::F :flag (:wat::rete::core::not ?b))])

(:wat::rete::defquery :ft::q :params [] :when [(?fact <- :ft::F)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              (:wat::rete::compile-all (:wat::core::PersistentVector (:ft::flip))
                (:wat::core::PersistentVector (:ft::q)))
              (:ft::F :flag true)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:ft::q)))))

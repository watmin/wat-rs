;; OVER THE CAP — 20 computed `bool` fields is 2^20 = 1_048_576 distinct facts, just past
;; `MAX_PROVABLE_FACT_POPULATION` (1_000_000). Provably FINITE and provably too large to
;; materialise: at the measured ~600 B/fact it is ~630 MB.
;;
;; This is the row that keeps the cap from being decorative. Without it the constant could be set
;; to anything — or removed — and every other test would still pass, which is the vacuous-gate
;; shape this arc keeps pulling out. The refusal here is CORRECT: "provably finite" is not the
;; same claim as "safe to admit", and the cap is where those two part company.
(:wat::core::defrecord :tl::Wide [f0 <- :wat::core::bool  f1 <- :wat::core::bool  f2 <- :wat::core::bool  f3 <- :wat::core::bool  f4 <- :wat::core::bool  f5 <- :wat::core::bool  f6 <- :wat::core::bool  f7 <- :wat::core::bool  f8 <- :wat::core::bool  f9 <- :wat::core::bool  f10 <- :wat::core::bool  f11 <- :wat::core::bool  f12 <- :wat::core::bool  f13 <- :wat::core::bool  f14 <- :wat::core::bool  f15 <- :wat::core::bool  f16 <- :wat::core::bool  f17 <- :wat::core::bool  f18 <- :wat::core::bool  f19 <- :wat::core::bool])

(:wat::rete::defrule :tl::flip
  :when  [(:tl::Wide (?b0 <- :f0) (?b1 <- :f1) (?b2 <- :f2) (?b3 <- :f3) (?b4 <- :f4) (?b5 <- :f5) (?b6 <- :f6) (?b7 <- :f7) (?b8 <- :f8) (?b9 <- :f9) (?b10 <- :f10) (?b11 <- :f11) (?b12 <- :f12) (?b13 <- :f13) (?b14 <- :f14) (?b15 <- :f15) (?b16 <- :f16) (?b17 <- :f17) (?b18 <- :f18) (?b19 <- :f19))]
  :then  [(:tl::Wide :f0 (:wat::rete::core::not ?b0) :f1 (:wat::rete::core::not ?b1) :f2 (:wat::rete::core::not ?b2) :f3 (:wat::rete::core::not ?b3) :f4 (:wat::rete::core::not ?b4) :f5 (:wat::rete::core::not ?b5) :f6 (:wat::rete::core::not ?b6) :f7 (:wat::rete::core::not ?b7) :f8 (:wat::rete::core::not ?b8) :f9 (:wat::rete::core::not ?b9) :f10 (:wat::rete::core::not ?b10) :f11 (:wat::rete::core::not ?b11) :f12 (:wat::rete::core::not ?b12) :f13 (:wat::rete::core::not ?b13) :f14 (:wat::rete::core::not ?b14) :f15 (:wat::rete::core::not ?b15) :f16 (:wat::rete::core::not ?b16) :f17 (:wat::rete::core::not ?b17) :f18 (:wat::rete::core::not ?b18) :f19 (:wat::rete::core::not ?b19))])

(:wat::rete::defquery :tl::q :params [] :when [(?fact <- :tl::Wide)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::rete::query
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert
            (:wat::rete::compile-all (:wat::core::PersistentVector (:tl::flip))
              (:wat::core::PersistentVector (:tl::q)))
            (:tl::Wide :f0 true :f1 true :f2 true :f3 true :f4 true :f5 true :f6 true :f7 true :f8 true :f9 true :f10 true :f11 true :f12 true :f13 true :f14 true :f15 true :f16 true :f17 true :f18 true :f19 true)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
        (:tl::q)))))

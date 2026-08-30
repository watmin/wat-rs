;; BOUNDARY, FAILING SIDE — one round short of what the workload needs, and it must be REFUSED.
;;
;; Identical to `..._boundary_pass.wat` except the cap is 501 instead of 502. If this ever passes,
;; the cap is off by one in the permissive direction; if its twin ever fails, off by one in the
;; strict direction — and the strict direction silently steals a round of legitimate depth.
(:wat::config::rete::set-max-fire-rounds! 501)
(:wat::core::defrecord :cap::Edge  [a <- :wat::core::i64  b <- :wat::core::i64])
(:wat::core::defrecord :cap::Start [n <- :wat::core::i64])
(:wat::core::defrecord :cap::Reach [n <- :wat::core::i64])

(:wat::rete::defrule :cap::seed
  :when [(:cap::Start (?n <- :n))]
  :then [(:cap::Reach :n ?n)])

;; The cyclic rule — Reach reads Reach — and legal, because `?y` is copied from Edge.
(:wat::rete::defrule :cap::step
  :when [(:cap::Reach (?x <- :n))
         (:cap::Edge (?x <- :a) (?y <- :b))]
  :then [(:cap::Reach :n ?y)])

(:wat::rete::defquery :cap::q :params [] :when [(?fact <- :cap::Reach)])

(:wat::core::defn :cap::edges [] -> (:wat::core::PersistentVector :- [:cap::Edge])
  (:wat::core::into (:wat::core::PersistentVector)
    (:wat::core::mapv
      (:wat::core::fn [i <- :wat::core::i64] -> :cap::Edge
        (:cap::Edge :a i :b (:wat::core::i64::+ i 1)))
      (:wat::core::range 0 500))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; ⛔ HAND-FACED, NOT CODEMOD'D (arc 278 the fire-outcome wall). The corpus codemod collapses both
  ;; ceiling arms to an `assertion-failed!` message — correct for a fixture that merely must not
  ;; proceed, and WRONG here: this gate exists to pin the refusal's `cap`, and a message string
  ;; throws that number away. It would still have gone red, then green on a careless "fix", while
  ;; proving only that *something* stopped. The arm is the assertion; print its fields.
  (:wat::core::match
    (:wat::rete::fire-rules
      (:wat::core::match (:wat::rete::insert
        (:wat::core::match (:wat::rete::insert-all
          (:wat::core::match (:wat::rete::compile-all
            (:wat::core::PersistentVector (:cap::seed) (:cap::step))
            (:wat::core::PersistentVector (:cap::q))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
          (:cap::edges)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
        (:cap::Start :n 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))
    ((:wat::rete::FireOutcome::Fired fired)
      ;; The permissive off-by-one: one round SHORT of the workload must NOT complete.
      (:wat::kernel::println
        (:wat::core::i64::to-string
          (:wat::core::length (:wat::rete::query fired (:cap::q))))))
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded limit used rounds)
      (:wat::core::do
        (:wat::kernel::println "ARM MemoryCeilingExceeded")
        (:wat::kernel::println limit)))
    ((:wat::rete::FireOutcome::RoundCapExceeded cap still-deriving)
      (:wat::core::do
        (:wat::kernel::println "ARM RoundCapExceeded")
        (:wat::kernel::println cap)
        (:wat::kernel::println still-deriving)))))

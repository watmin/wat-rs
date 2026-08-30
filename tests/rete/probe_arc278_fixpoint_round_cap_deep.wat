;; DEEP BUT TERMINATING — must SUCCEED. The false-positive guard for BOTH gates: the termination
;; verifier must accept it at compile, and the round cap must not fire on it at run.
;;
;; Single-source reachability along a 500-edge path. `Reach(y) :- Reach(x), Edge(x,y)` derives one
;; new node per round, so this runs EXACTLY 502 ROUNDS and stops at 501 facts.
;;
;; ⚠ 502, MEASURED — not 500, and not a round number chosen by feel. Bisecting the cap shows it
;; fails at 501 and passes at 502: the extra two are the seed rule's round plus the final no-op
;; round that proves convergence. The path length 500 is arbitrary in itself; what is NOT arbitrary
;; is that its round count is pinned, by `..._boundary_pass.wat` / `..._boundary_fail.wat`, which
;; run this same workload at cap 502 and 501. A fixture whose depth is merely "comfortably under
;; the cap" tests nothing about the cap's edge.
;;
;; ⚠ IT IS RANGE-RESTRICTED, AND THAT IS THE WHOLE POINT. `y` is COPIED out of `Edge`, never
;; computed, so the fact domain stays finite and the verifier can PROVE termination even though
;; the rule is plainly cyclic (Reach feeds Reach). An earlier version of this fixture used a
;; guarded counter — `N(k+1) :- N(k), (where (< ?k 500))` — which terminates but is NOT provable:
;; the verifier would have to reason about monotonicity and comparison direction against a
;; literal. It was correctly refused, and replaced by this, which is the honest shape of "deep
;; workload" anyway: transitive reachability is what real deep Datalog looks like, not a counter.
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
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              (:wat::core::match (:wat::rete::insert-all
                (:wat::core::match (:wat::rete::compile-all
                  (:wat::core::PersistentVector (:cap::seed) (:cap::step))
                  (:wat::core::PersistentVector (:cap::q))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
                (:cap::edges)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
              (:cap::Start :n 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:cap::q))))))

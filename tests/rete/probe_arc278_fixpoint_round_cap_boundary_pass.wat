;; BOUNDARY, PASSING SIDE — the deep workload with the cap set to EXACTLY its round count.
;;
;; The workload is 500 edges and takes 502 rounds — MEASURED by bisecting the cap, not assumed.
;; (It is 502 rather than 500 because of the seed rule's round plus the final no-op round that
;; proves convergence; the two are invisible until you pin them, which is the whole argument for
;; a boundary test over a round number.)
;;
;; Paired with `..._boundary_fail.wat`, which is this file with the cap ONE LOWER and must be
;; refused. Together they pin the off-by-one that every cap risks: a `>` where a `>=` belongs
;; silently costs one round of legitimate depth, and no amount of "500 is comfortably under
;; 10,000" testing would ever notice.
(:wat::config::rete::set-max-fire-rounds! 502)
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

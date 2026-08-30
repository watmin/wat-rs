;; THE FN-HEADED EXPLOIT — a rule that mints a novel fact every round with the computation hidden
;; one level down, inside a RETE fn. Must be REFUSED at compile.
;;
;; This is the hole 4.2 shipped with, named in its own doctrine as "admitted, not proven". It was
;; then demonstrated: this file compiled clean and ran to the round cap, deriving forever.
;;
;; WHY IT SLIPPED: the verifier inspected the `:then` ITEM only, and `(:fm::bump ?n)`'s arguments
;; are bare bound variables — nothing computed in sight. The minting happens in `bump`'s body.
;;
;; TWO THINGS HAD TO BE RIGHT TO EVEN WRITE IT, and getting either wrong made the probe pass for
;; the wrong reason:
;;   1. `:wat::rete::core::defn`, NOT `:wat::core::defn`. A plain defn is refused as a `:then` head
;;      by `then-item-fence`'s Law A conjunct ("is not a rete primitive"). Three earlier attempts
;;      used the plain door, failed for that unrelated reason, and briefly convinced me the hole
;;      was already guarded.
;;   2. The total fallback spelling `(:wat::rete::core::i64::+ a b :undefined 0)`. The bare
;;      `:wat::core::i64::+` is refused by the same fence as "not total".
(:wat::core::defrecord :fm::N [k <- :wat::core::i64])

;; The body constructs from a COMPUTED value. Note `(:fm::N :k …)` is kwargs SUGAR — by the time
;; this is a stored fn body the macro has rewritten it to `:wat::core::kwargs-construct`, which is
;; exactly the head the analysis has to know about.
(:wat::rete::core::defn :fm::bump
  [n <- :fm::N]
  -> :fm::N
  (:fm::N :k (:wat::rete::core::i64::+ (:fm::N/k n) 1 :undefined 0)))

(:wat::rete::defrule :fm::grow
  :when [(?n <- :fm::N)]
  :then [(:fm::bump ?n)])

(:wat::rete::defquery :fm::q :params [] :when [(?fact <- :fm::N)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; ⛔ THE COMPILE MATCH IS HOISTED AND ITS ARM PRINTS — hand-faced, NOT codemod'd. The
  ;; corpus codemod collapses `MayNotTerminate` to an `assertion-failed!` message, which is
  ;; right for a fixture that merely must not proceed and WRONG here: this gate exists to
  ;; pin the verdict's `rule` and `fact-type`, and a message string throws both away.
  (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:fm::grow))
                (:wat::core::PersistentVector (:fm::q)))
    ((:wat::rete::CompileOutcome::Compiled __session)
      ;; No println before compile-all: an earlier version of this fixture announced "compiled" FIRST,
  ;; which prints whether or not the compile then fails, and cost real time reading a verdict that
  ;; was never a verdict.
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              __session
              (:fm::N :k 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:fm::q))))))
    ((:wat::rete::CompileOutcome::MayNotTerminate rule fact-type)
      (:wat::core::do
        (:wat::kernel::println "ARM MayNotTerminate")
        (:wat::kernel::println rule)
        (:wat::kernel::println fact-type)))))


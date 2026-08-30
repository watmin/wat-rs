;; RUNAWAY — must be REFUSED by the TERMINATION VERIFIER, at compile, before a fact is inserted.
;;
;; `N(k)` derives `N(k+1)` with no guard, so every round mints a structurally novel fact and the
;; dedup that bounds a Datalog fixpoint never bites. This is the exact shape
;; `DESIGN-STONE-4b-cascade-fixpoint` NAMED and deferred a cap for ("let need reveal"); the need
;; revealed. Before the cap (2026-08-27) this died on
;; `memory allocation of 545259536 bytes failed` — no wat error, no span, no rule named, and with
;; no ulimit that is the machine's memory.
;;
;; Its twin `probe_arc278_fixpoint_round_cap_deep.wat` is deep (502 rounds) and CYCLIC and must
;; still be accepted — because its head is COPIED from a body binding rather than computed. The
;; pair is the test: refuse unbounded derivation without refusing depth.
(:wat::core::defrecord :cap::N [k <- :wat::core::i64])

(:wat::rete::defrule :cap::grow
  :when [(:cap::N (?k <- :k))]
  :then [(:cap::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defquery :cap::q :params [] :when [(?fact <- :cap::N)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; ⛔ THE COMPILE MATCH IS HOISTED AND ITS ARM PRINTS — hand-faced, NOT codemod'd. The
  ;; corpus codemod collapses `MayNotTerminate` to an `assertion-failed!` message, which is
  ;; right for a fixture that merely must not proceed and WRONG here: this gate exists to
  ;; pin the verdict's `rule` and `fact-type`, and a message string throws both away.
  (:wat::core::match (:wat::rete::compile-all
                (:wat::core::PersistentVector (:cap::grow))
                (:wat::core::PersistentVector (:cap::q)))
    ((:wat::rete::CompileOutcome::Compiled __session)
      (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::core::match (:wat::rete::fire-rules
            (:wat::core::match (:wat::rete::insert
              __session
              (:cap::N :k 0)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
          (:cap::q))))))
    ((:wat::rete::CompileOutcome::MayNotTerminate rule fact-type)
      (:wat::core::do
        (:wat::kernel::println "ARM MayNotTerminate")
        (:wat::kernel::println rule)
        (:wat::kernel::println fact-type)))))


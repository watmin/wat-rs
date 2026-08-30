;; THE INSERT DOOR of the per-session memory ceiling — staging, with no fire at all.
;;
;; ⛔ THIS FIXTURE NEVER CALLS `fire-rules`. That is the entire point. The ceiling was once checked
;; only inside the fixpoint, which made it a FIRE ceiling wearing a SESSION ceiling's name —
;; measured 2026-08-29: **2_500_000 facts staged with no fire reached 4.0 GB against a 1 GiB
;; contract, with no diagnostic.** A session grows through TWO doors and the contract is one
;; contract, so each door needs its own proof; a fixture that inserts AND fires cannot tell you
;; which one refused.
;;
;; Driven at the ceiling's floor (4096) for its sibling's stated reason: the shape that genuinely
;; needs this — millions of staged facts — takes minutes and gigabytes, so the ceiling is lowered
;; until an honest workload crosses it. What is proven is the MECHANISM: staging is counted, the
;; boundary is checked on every insert, and the refusal is a located diagnostic naming the `insert`
;; call rather than an allocator abort.
(:wat::config::rete::set-max-session-bytes! 4096)

(:wat::core::defrecord :ins::Edge [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::rete::defrule :ins::noop
  :when [(:ins::Edge (?a <- :a))]
  :then [])

(:wat::core::defn :ins::seed
  [s <- :wat::rete::Session  n <- :wat::core::i64]
  -> :wat::rete::InsertOutcome
  ;; ⛔ THE FOLD CARRIES THE OUTCOME, NOT A SESSION — hand-faced, NOT codemod'd. The corpus codemod
  ;; unwraps each `insert` into a Session and dies loudly on a ceiling, which is right for a fixture
  ;; that merely must not proceed. It is WRONG here: this gate exists to pin `limit`, `used` and
  ;; `staged`, and an `assertion-failed!` throws all three away. The fold therefore SHORT-CIRCUITS
  ;; on the ceiling arm and hands it back intact for `:user::main` to print.
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::InsertOutcome  i <- :wat::core::i64] -> :wat::rete::InsertOutcome
      (:wat::core::match acc
        ;; still staging — try the next fact
        ((:wat::rete::InsertOutcome::Inserted session)
          (:wat::rete::insert session (:ins::Edge :a i :b (:wat::core::i64::+ i 1))))
        ;; already breached — carry the FIRST breach through UNCHANGED (`acc` itself, not a rebuilt
        ;; copy). Re-inserting after a ceiling would report whichever fact happened to be last
        ;; rather than the one that crossed it, and rebuilding the variant would be three chances
        ;; to transcribe a field wrong for no gain.
        ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __s) acc)))
    (:wat::rete::InsertOutcome::Inserted s)
    (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :ins)
     s     (:wat::rete::compile-all rules (:wat::core::PersistentVector))]
    (:wat::core::match (:ins::seed s 200000)
      ;; Staging 200_000 facts under a 4096-byte ceiling must NOT reach here.
      ((:wat::rete::InsertOutcome::Inserted staged)
        (:wat::kernel::println (:wat::core::length (:wat::rete::Session/facts staged))))
      ((:wat::rete::InsertOutcome::MemoryCeilingExceeded limit used staged)
        (:wat::core::do
          (:wat::kernel::println "ARM MemoryCeilingExceeded")
          (:wat::kernel::println limit)
          (:wat::kernel::println used)
          (:wat::kernel::println staged))))))

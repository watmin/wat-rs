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

(:wat::core::defn :ins::seed [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert acc (:ins::Edge :a i :b (:wat::core::i64::+ i 1))))
    s (:wat::core::range 0 200000)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :ins)
     s     (:wat::rete::compile-all rules (:wat::core::PersistentVector))
     s     (:ins::seed s 200000)]
    (:wat::kernel::println (:wat::core::length (:wat::rete::Session/facts s)))))

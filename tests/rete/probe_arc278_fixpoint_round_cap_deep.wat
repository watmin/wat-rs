;; DEEP BUT TERMINATING — must SUCCEED. The false-positive guard for the round cap.
;;
;; IDENTICAL to `probe_arc278_fixpoint_round_cap.wat` except for the one `:where` below. It runs
;; 500 ROUNDS — ten times the deepest thing in the grid (`deep-cascade` at depth 50) — and
;; terminates, deriving N(0..500).
;;
;; This is the row that makes the cap honest rather than merely loud: a cap that fired here would
;; be capping DEPTH, and depth is a legitimate workload shape. One `:where` apart, opposite
;; verdicts.
(:wat::core::defrecord :cap::N [k <- :wat::core::i64])

(:wat::rete::defrule :cap::grow
  :when [(:cap::N (?k <- :k))
         (:wat::rete::where (:wat::rete::core::i64::< ?k 500))]
  :then [(:cap::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])

(:wat::rete::defquery :cap::q :params [] :when [(?fact <- :cap::N)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::rete::fire-rules
            (:wat::rete::insert
              (:wat::rete::compile-all
                (:wat::core::PersistentVector (:cap::grow))
                (:wat::core::PersistentVector (:cap::q)))
              (:cap::N :k 0)))
          (:cap::q))))))

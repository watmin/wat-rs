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
  (:wat::kernel::println
    (:wat::core::i64::to-string
      (:wat::core::length
        (:wat::rete::query
          (:wat::rete::fire-rules
            (:wat::rete::insert
              (:wat::rete::insert-all
                (:wat::rete::compile-all
                  (:wat::core::PersistentVector (:cap::seed) (:cap::step))
                  (:wat::core::PersistentVector (:cap::q)))
                (:cap::edges))
              (:cap::Start :n 0)))
          (:cap::q))))))

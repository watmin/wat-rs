;; NOTE-rete-a-where-before-a-fact-condition-silently-matches-nothing.md (2026-08-24, inbound).
;; A `where` FOLLOWED by a fact condition is claimed to match nothing, silently.
(:wat::core::defrecord :wb::A [n <- :wat::core::i64])
(:wat::core::defrecord :wb::B [m <- :wat::core::i64])
(:wat::core::defrecord :wb::Out [n <- :wat::core::i64])

;; WHERE FIRST — the reported defect.
(:wat::rete::defrule :wb::where-first
  :when
  [(:wb::A (?n <- :n))
   (:wat::rete::where (:wat::rete::core::i64::> ?n 5))
   (:wb::B (?m <- :m))]
  :then [(:wb::Out :n ?n)])

(:wat::rete::defquery :wb::q :params [] :when [(?fact <- :wb::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :wb)
         session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wb::q)))
         session (:wat::rete::insert session (:wb::A :n 10))
         session (:wat::rete::insert session (:wb::B :m 1))
         fired   (:wat::rete::fire-rules session)]
        (:wat::rete::query fired (:wb::q))))))

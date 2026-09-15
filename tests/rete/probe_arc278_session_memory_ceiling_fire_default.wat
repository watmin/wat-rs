;; THE NON-VACUITY TWIN of `probe_arc278_session_memory_ceiling.wat` — the IDENTICAL fanout
;; workload with NO ceiling directive, so it runs at the 1 GiB default and must COMPLETE.
;;
;; ⛔ WITHOUT THIS FILE THE FIRE-DOOR GATE IS SATISFIED BY A CEILING OF ZERO, or by a check that
;; refuses before doing any work. It is the row that makes "refused at 16 MiB" mean something: the
;; same 40_000-fact derivation is a legitimate workload the substrate must run, and only the
;; configured ceiling separates the two outcomes. Bisected 2026-08-29: this workload completes from
;; 64 MiB upward, so the default has ~64x of headroom over it.
;;
;; Keep the two files' rules IDENTICAL. If you change one, change both — their whole evidential
;; value is that the ONLY difference is the ceiling.

(:wat::core::defrecord :fd::A [a <- :wat::core::i64])
(:wat::core::defrecord :fd::B [b <- :wat::core::i64])
(:wat::core::defrecord :fd::C [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::rete::defrule :fd::cross
  :when [(:fd::A (?x <- :a)) (:fd::B (?y <- :b))]
  :then [(:fd::C :a ?x :b ?y)])

(:wat::rete::defquery :fd::q :params [] :when [(?fact <- :fd::C)])

(:wat::core::defn :fd::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert (:wat::rete::insert acc (:fd::A :a i)) (:fd::B :b i)))
    s (:wat::core::range 0 200)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :fd)
     s     (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fd::q)))
     s     (:fd::seed s)
     f     (:wat::rete::fire-rules s)]
    (:wat::kernel::println (:wat::core::length (:wat::rete::query f (:fd::q))))))

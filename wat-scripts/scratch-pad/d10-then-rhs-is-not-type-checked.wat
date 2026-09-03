;; d10-then-rhs-is-not-type-checked.wat — work-list D10.
;;
;; THE RETE `:then` RHS DOES NOT TYPE-CHECK ITS FIELD VALUES, WHILE THE REST OF THE LANGUAGE DOES.
;;
;;   ordinary construction   (:td::Bad :n "x")  ->  #wat.check/TypeMismatch
;;                              ":td::Bad: parameter #1 expects :wat::core::i64; got :wat::core::String"
;;   the SAME construction inside `:then`       ->  compiles, fires, and the derived fact is
;;                              #tr/Bad {:n "not-an-i64"}
;;
;; Driven 2026-09-02 for BOTH a bound `?var` (below) and a literal. The RHS walls that DO exist are
;; RhsArityMismatch / RhsMissingFields / RhsPositionalConstructionRetired / RhsUnresolvableOperand —
;; every one structural. None types a value.
;;
;; ⚠ NOT a parametric-record problem. `:tr::Box.s` is concretely `:wat::core::String`. The `:when`
;; side DOES reason about types — a comparison on an erased `:T` is refused with
;; ConstraintTypeNotComparable — so this is the `:then` surface specifically.
;;
;; ⛔ THE CONTROL IS LOAD-BEARING AND WAS ADDED AFTER THREE VACUOUS PROBES. `FireOutcome::Fired`
;; means the fire COMPLETED, not that a rule produced anything — a probe whose `collect-rules`
;; names an empty namespace compiles ZERO rules and still reports Fired. `Good count: 1` is what
;; proves this probe is live; without it every number below is meaningless.
;; ANTI-VACUITY: the well-typed rule `ok` must derive, or this probe proves nothing.
(:wat::core::defrecord :tr::Box [k <- :wat::core::i64  s <- :wat::core::String])
(:wat::core::defrecord :tr::Good [n <- :wat::core::i64])
(:wat::core::defrecord :tr::Bad  [n <- :wat::core::i64])

(:wat::rete::defrule :tr::ok
  :when [(:tr::Box (?k <- :k))]
  :then [(:tr::Good :n ?k)])                    ;; i64 into i64 — the CONTROL

(:wat::rete::defrule :tr::bad
  :when [(:tr::Box (?k <- :k) (?s <- :s))]
  :then [(:tr::Bad :n ?s)])                     ;; String binding into i64 — the SUBJECT

(:wat::rete::defquery :tr::qg :params [] :when [(?f <- :tr::Good)])
(:wat::rete::defquery :tr::qb :params [] :when [(?f <- :tr::Bad)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [s0 (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :tr)
                             (:wat::core::PersistentVector (:tr::qg) (:tr::qb)))
          ((:wat::rete::CompileOutcome::Compiled __s) __s)
          ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "mnt" :wat::core::None :wat::core::None)))
     s1 (:wat::core::match (:wat::rete::insert s0 (:tr::Box :k 7 :s "not-an-i64"))
          ((:wat::rete::InsertOutcome::Inserted __x) __x)
          ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::assertion-failed! "c" :wat::core::None :wat::core::None)))]
    (:wat::core::match (:wat::rete::fire-rules s1)
      ((:wat::rete::FireOutcome::Fired __f)
        (:wat::core::do
          (:wat::kernel::println "CONTROL Good count:")
          (:wat::kernel::println (:wat::core::length (:wat::rete::query __f (:tr::qg))))
          (:wat::kernel::println "SUBJECT Bad count:")
          (:wat::kernel::println (:wat::core::length (:wat::rete::query __f (:tr::qb))))
          (:wat::kernel::println (:wat::rete::query __f (:tr::qb)))))
      ((:wat::rete::FireOutcome::MemoryCeilingExceeded __a __b __c) (:wat::kernel::println "ceil"))
      ((:wat::rete::FireOutcome::RoundCapExceeded __a __b) (:wat::kernel::println "roundcap")))))

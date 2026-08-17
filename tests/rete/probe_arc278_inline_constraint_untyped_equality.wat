;; NEGATIVE FIXTURE — an UNTYPED inline alpha constraint using generic EQUALITY — the op the discrimination tree keys on.
;;
;; `(:wat::core::= :value 42)` sits inside a fact pattern, where `classify_rete_clause`
;; (matcher.rs:331) makes `Constraint` a sibling of `Bind`. `compile-condition` (wat/rete.wat:679)
;; has no branch for it, so law A never sees it and this compiles + fires TODAY.
;;
;; Generic `>` routes through `compare_values`, whose `?` propagates the incomparable-operands
;; error — the domain hole the per-type surface exists to delete.
;;
;; MUST BE REFUSED once the grammar forces the per-type rete spelling.
;; See DESIGN-STONE-inline-constraint-admits-non-rete.md.

(:wat::core::defrecord :probe::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :probe::Hot     [location <- :wat::core::String])

(:wat::rete::defrule :probe::untyped-equality
  :when
  [(:probe::Reading (?loc <- :location) (:wat::core::= :value 42))]
  :then
  [(:probe::Hot :location ?loc)])

(:wat::rete::defquery :probe::q-Hot
  :params []
  :when [(?fact <- :probe::Hot)])


(:wat::core::defn :probe::run [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :probe)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q-Hot)))
     session (:wat::rete::insert session (:probe::Reading :location "Oslo"   :value 42))
     session (:wat::rete::insert session (:probe::Reading :location "Bergen" :value 3))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:probe::q-Hot)))))

;; NEGATIVE FIXTURE — an UNTYPED inline alpha constraint using generic EQUALITY — the op the discrimination tree keys on.
;;
;; `(:wat::core::= :value 42)` sits inside a fact pattern. Freeze wall
;; (`validate.rs` CoreGeneric → NonReteConstraint) and intern
;; `compile_condition_local` refuse it. Law A sees this spelling.
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

;; POSITIVE FIXTURE — per-type `i64::>` inline constraint. GREEN admit: compiles, fires, prunes.
;;
;; `(:wat::rete::i64::> :value 10)` sits inside a fact pattern.
;; `classify_rete_clause` (`clause.rs:173`) names the per-type rete spelling;
;; `compile-condition` (`wat/rete/compile.wat:364`) admits it. Discriminates
;; Oslo only, never Bergen.
;; See DESIGN-STONE-inline-constraint-admits-non-rete.md.

(:wat::core::defrecord :probe::Reading [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :probe::Hot     [location <- :wat::core::String])

(:wat::rete::defrule :probe::per-type-ordering
  :when
  [(:probe::Reading (?loc <- :location) (:wat::rete::i64::> :value 10))]
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

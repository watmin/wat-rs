;; Fixture for probe_ex003_construction_annotation_is_honoured.rs — a `:then` item whose NESTED
;; operand constructs with `:- []`: the rete :then validator (`src/rete/validate`) must peel the
;; spec from the lowered `kwargs-construct`, not report `:rt::Inner` has no field `:-`.
(:wat::core::defrecord :rt::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :rt::Inner  [n <- :wat::core::i64])
(:wat::core::defrecord :rt::Outer  [inner <- :rt::Inner])

(:wat::rete::defrule :rt::gather
  :when [(:rt::Anchor (?x :- :x))]
  :then [(:rt::Outer :inner (:rt::Inner :- [] :n ?x))])

(:wat::rete::defquery :rt::q
  :params []
  :when [(:rt::Outer (?i :- :inner))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::rete::compile-all
    (:wat::rete::collect-rules :rt)
    (:wat::core::PersistentVector (:rt::q)))
    [:wat::rete::CompileOutcome.Compiled {:session s}
      (:wat::core::match (:wat::rete::insert s (:rt::Anchor :x 4))
        [:wat::rete::InsertOutcome.Inserted {:session s2}
          (:wat::core::match (:wat::rete::fire-rules s2)
            [:wat::rete::FireOutcome.Fired {:value s3}
              (:wat::kernel::println (:wat::core::length (:wat::rete::query s3 (:rt::q))))]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::println "mem")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::println "cap")])]
        [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::println "mem")])]
    [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::println "may-not-terminate")]))

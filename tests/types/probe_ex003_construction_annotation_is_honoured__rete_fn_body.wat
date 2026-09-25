;; Fixture for probe_ex003_construction_annotation_is_honoured.rs — the rete consumers of
;; `kwargs-construct` (`src/rete/expr_ir` lowering) now receive the spec the expander used to strip.
;; At HEAD this ran; with the expander change but no peel in `lower_construct` it died with
;; `unknown aggregate :rn::Rate`. Must print the one derived fact's `?count`.
(:wat::core::defrecord :rn::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :rn::Rate   [count <- :wat::core::i64])

(:wat::rete::core::defn :rn::mk [k <- :wat::core::i64] -> :rn::Rate
  (:rn::Rate :- [] :count k))

(:wat::rete::defrule :rn::gather
  :when [(:rn::Anchor (?x :- :x))]
  :then [(:rn::mk 5)])

(:wat::rete::defquery :rn::q-Rate
  :params []
  :when [(:rn::Rate (?count :- :count))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::rete::compile-all
    (:wat::rete::collect-rules :rn)
    (:wat::core::PersistentVector (:rn::q-Rate)))
    [:wat::rete::CompileOutcome.Compiled {:session s}
      (:wat::core::match (:wat::rete::insert s (:rn::Anchor :x 0))
        [:wat::rete::InsertOutcome.Inserted {:session s2}
          (:wat::core::match (:wat::rete::fire-rules s2)
            [:wat::rete::FireOutcome.Fired {:value s3}
              (:wat::kernel::println
                (:wat::core::Option/expect
                  (:wat::map::get (:wat::core::first (:wat::rete::query s3 (:rn::q-Rate))) "?count")
                  "q-Rate: ?count"))]
            [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::println "mem")]
            [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::println "cap")])]
        [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::println "mem")])]
    [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::println "may-not-terminate")]))

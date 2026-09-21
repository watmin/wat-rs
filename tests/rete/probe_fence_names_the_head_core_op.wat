;; tests/rete/probe_fence_names_the_head_core_op.wat — total-but-not-rete where world for the
;; probe_fence_names_the_head RED probe. `(:wat::i64::> ?c 0)` is pure, deterministic, AND
;; total, but it is a core spelling — Law A (rete-primitive) must be named, not :total.
;; Mirrors tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat's shape exactly, swapping
;; the violating verb. A regression that reports this as "is not total" stays red.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c :- :celsius))
   (:wat::rete::where (:wat::i64::> ?c 0))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::rete::defquery :wf::q-Gate
  :params []
  :when [(?fact :- :wf::Gate)])


(:wat::core::defn :user::run-gate-c5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wf)
       session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wf::q-Gate))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius 5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
      (:wat::rete::query fired (:wf::q-Gate)))))

;; tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat — comparison-gate world for the where_oracle probe;
;; loaded via startup_from_file. Rule filters Temperature by (where (> ?c 0)).

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wg::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wg::cold-gate
  :when
  [(:weather::Temperature (?c :- :celsius))
   (:wat::rete::where (:wat::rete::i64::> ?c 0))]
  :then
  [(:wg::Gate :celsius ?c)])

(:wat::rete::defquery :wg::q-Gate
  :params []
  :when [(?fact :- :wg::Gate)])


;; 1 — the where PASSES: Temp(5), (> 5 0) true → exactly one Gate derived.
(:wat::core::defn :user::run-gate-c5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wg)
       session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wg::q-Gate))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius 5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
      (:wat::rete::query fired (:wg::q-Gate)))))

;; 2 — the where BLOCKS: Temp(-5), (> -5 0) false → zero Gates (the filter actually filters).
(:wat::core::defn :user::run-gate-cneg5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wg)
       session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wg::q-Gate))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
      (:wat::rete::query fired (:wg::q-Gate)))))


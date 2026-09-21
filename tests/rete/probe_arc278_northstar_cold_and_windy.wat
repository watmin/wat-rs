;; tests/rete/probe_arc278_northstar_cold_and_windy.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). The north-star defrule: cold-and-windy end-to-end DSL spec.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature
     (?loc :- :location)
     (?c   :- :celsius)
     (:wat::rete::i64::< ?c 20))
   (:weather::WindSpeed
     (?loc :- :location)
     (?k   :- :kph)
     (:wat::rete::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy :location ?loc)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact :- :weather::ColdAndWindy)])


;; The lifecycle, value-threaded: collect → compile → insert → insert → fire → query, then COUNT the
;; derived facts (wrapped in `length` so the Rust driver just-evals to a scalar).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules    (:wat::rete::collect-rules :weather)
       session  (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:weather::q-ColdAndWindy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session  (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius 15 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       session  (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 45 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       fired    (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
      (:wat::rete::query fired (:weather::q-ColdAndWindy)))))


;; tests/rete/probe_arc278_P12_explain_walk.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade + two defrules for explain-walk tests.

(:wat::core::defrecord :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])
(:wat::core::defrecord :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc :- :location) (?c :- :celsius) (:wat::rete::i64::< ?c 0))
   (:weather::WindSpeed   (?loc :- :location) (?k :- :kph)     (:wat::rete::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy ?c ?k)])

(:wat::rete::defrule :weather::alert
  :when
  [(:weather::ColdAndWindy (?c :- :celsius) (?k :- :kph))]
  :then
  [(:weather::WeatherAlert :celsius ?c :kph ?k)])

;; LEVEL 1 — explain a directly-derived fact reaches its two input facts. `ColdAndWindy` is derived by
;; `cold-and-windy` from `Temperature` ⋈ `WindSpeed`; its why-tree's `:via` has exactly those two supporting
;; facts → length 2.
(:wat::core::defn :user::explain-coldandwindy-via-length [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::DerivationNode/via
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :weather)
         session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
         session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         session (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         fired   (:wat::core::match (:wat::rete::fire-rules-explain session) [:wat::rete::FireOutcome.Fired {:value __explained} __explained] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules-explain: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules-explain: fixpoint round cap exceeded")])]
        (:wat::rete::explain fired (:weather::ColdAndWindy :celsius -5 :kph 40))))))

;; LEVEL 2 — explain a CASCADE-derived fact: `WeatherAlert` is derived by `alert` from the derived
;; `ColdAndWindy`. Its `:via` has exactly one supporting fact (the ColdAndWindy).
(:wat::core::defn :user::explain-weatheralert-via-length [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::DerivationNode/via
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :weather)
         session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
         session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         session (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
         fired   (:wat::core::match (:wat::rete::fire-rules-explain session) [:wat::rete::FireOutcome.Fired {:value __explained} __explained] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules-explain: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules-explain: fixpoint round cap exceeded")])]
        (:wat::rete::explain fired (:weather::WeatherAlert :celsius -5 :kph 40))))))


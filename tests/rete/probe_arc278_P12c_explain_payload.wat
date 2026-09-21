;; tests/rete/probe_arc278_P12c_explain_payload.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade for explain-payload tests.

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

;; ── explain-payload probes ────────────────────────────────────────────────────
;; Shared prefix is :user::explain-cw-root / :user::explain-cw-step0; each field
;; probe is a one-line accessor over that node.

(:wat::core::defn :user::explain-cw-root [] -> :wat::rete::DerivationNode
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     session (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     session (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     ex      (:wat::core::match (:wat::rete::fire-rules-explain session) [:wat::rete::FireOutcome.Fired {:value __explained} __explained] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules-explain: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules-explain: fixpoint round cap exceeded")])]
    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))))

(:wat::core::defn :user::explain-cw-step0 [] -> :wat::rete::DerivationStep
  (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via (:user::explain-cw-root)) 0) "via[0]"))

(:wat::core::defn :user::explain-cw-via-length [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::DerivationNode/via (:user::explain-cw-root))))

(:wat::core::defn :user::step-pattern [] -> :wat::core::String
  (:wat::rete::DerivationStep/pattern (:user::explain-cw-step0)))

(:wat::core::defn :user::step-bindings-c [] -> (:wat::core::Option :- [:wat::core::Value])
  (:wat::map::get (:wat::rete::DerivationStep/bindings (:user::explain-cw-step0)) "?c"))

(:wat::core::defn :user::derived-node-rule [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::rete::DerivationNode/rule (:user::explain-cw-root)))

(:wat::core::defn :user::base-node-rule [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::rete::DerivationNode/rule (:wat::rete::DerivationStep/supporting (:user::explain-cw-step0))))

(:wat::core::defn :user::step-constraints-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::DerivationStep/constraints (:user::explain-cw-step0))))

(:wat::core::defn :user::step-constraint-0 [] -> :wat::WatAST
  (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationStep/constraints (:user::explain-cw-step0)) 0) "constraints[0]"))

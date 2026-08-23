;; tests/rete/probe_arc278_P12c_explain_payload.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade for explain-payload tests.

(:wat::core::defrecord :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])
(:wat::core::defrecord :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::core::i64::< ?c 0))
   (:weather::WindSpeed   (?loc <- :location) (?k <- :kph)     (:wat::rete::core::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy ?c ?k)])

(:wat::rete::defrule :weather::alert
  :when
  [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]
  :then
  [(:weather::WeatherAlert :celsius ?c :kph ?k)])

;; ── explain-payload probes ────────────────────────────────────────────────────
;; Lifecycle prefix binding `root` (explain of ColdAndWindy) and `step0` (its first via edge),
;; then a distinct tail per assertion — one named entry per test (the shared prefix is pure and
;; cheap to recompute per call).

(:wat::core::defn :user::step-pattern [] -> :wat::core::String
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))
     step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
    (:wat::rete::DerivationStep/pattern step0)))

(:wat::core::defn :user::step-bindings-c [] -> (:wat::core::Option :- [:wat::core::Value])
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))
     step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
    (:wat::core::PersistentMap/get (:wat::rete::DerivationStep/bindings step0) "?c")))

(:wat::core::defn :user::derived-node-rule [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))]
    (:wat::rete::DerivationNode/rule root)))

(:wat::core::defn :user::base-node-rule [] -> (:wat::core::Option :- [:wat::core::String])
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))
     step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
    (:wat::rete::DerivationNode/rule (:wat::rete::DerivationStep/supporting step0))))

(:wat::core::defn :user::step-constraints-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))
     step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
    (:wat::core::length (:wat::rete::DerivationStep/constraints step0))))

(:wat::core::defn :user::step-constraint-0 [] -> :wat::WatAST
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :weather)
     session (:wat::rete::compile rules)
     session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
     session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
     ex      (:wat::rete::fire-rules-explain session)
     root    (:wat::rete::explain ex (:weather::ColdAndWindy :celsius -5 :kph 40))
     step0   (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationNode/via root) 0) "via[0]")]
    (:wat::core::Option/expect (:wat::core::get (:wat::rete::DerivationStep/constraints step0) 0) "constraints[0]")))


;; tests/rete/probe_arc278_P12_explain_walk.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade + two defrules for explain-walk tests.

(:wat::core::defrecord :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])
(:wat::core::defrecord :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 0))
   (:weather::WindSpeed   (?loc <- :location) (?k <- :kph)     (:wat::rete::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy ?c ?k)])

(:wat::rete::defrule :weather::alert
  :when
  [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]
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
         session (:wat::rete::compile rules)
         session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
         session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
         fired   (:wat::rete::fire-rules-explain session)]
        (:wat::rete::explain fired (:weather::ColdAndWindy :celsius -5 :kph 40))))))

;; LEVEL 2 — explain a CASCADE-derived fact: `WeatherAlert` is derived by `alert` from the derived
;; `ColdAndWindy`. Its `:via` has exactly one supporting fact (the ColdAndWindy).
(:wat::core::defn :user::explain-weatheralert-via-length [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::DerivationNode/via
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :weather)
         session (:wat::rete::compile rules)
         session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
         session (:wat::rete::insert session (:weather::WindSpeed    :kph 40 :location "Oslo"))
         fired   (:wat::rete::fire-rules-explain session)]
        (:wat::rete::explain fired (:weather::WeatherAlert :celsius -5 :kph 40))))))


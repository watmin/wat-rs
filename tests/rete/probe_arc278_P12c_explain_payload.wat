;; tests/rete/probe_arc278_P12c_explain_payload.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade for explain-payload tests.

(:wat::core::defrecord :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])
(:wat::core::defrecord :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 0))
   (:weather::WindSpeed   (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]
  :then
  (:wat::rete::insert (:weather::ColdAndWindy ?c ?k)))

(:wat::rete::defrule :weather::alert
  :when
  [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]
  :then
  (:wat::rete::insert (:weather::WeatherAlert ?c ?k)))


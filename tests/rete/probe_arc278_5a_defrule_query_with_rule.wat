;; tests/rete/probe_arc278_5a_defrule_query_with_rule.wat — records + defrule fixture for the
;; probe_arc278_5a_defrule_query probe; loaded via startup_from_file for the defrule tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
   (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]
  :then
  (:wat::rete::insert (:weather::ColdAndWindy :location ?loc)))

;; tests/rete/probe_arc278_5b_collect_rules.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two weather defrules, one non-rule defn, one other-ns defrule.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::rete::defrule :weather::cold-and-windy
  :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
         (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]
  :then (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
(:wat::rete::defrule :weather::cold-temp
  :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 0))]
  :then (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
(:wat::core::defn :weather::helper [] -> :wat::core::i64 42)
(:wat::rete::defrule :other::windy
  :when [(:weather::WindSpeed (?loc <- :location) (?k <- :kph))]
  :then (:wat::rete::insert (:weather::ColdAndWindy ?loc)))

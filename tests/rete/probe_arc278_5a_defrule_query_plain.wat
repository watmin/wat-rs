;; tests/rete/probe_arc278_5a_defrule_query_plain.wat — records-only fixture (no defrule) for the
;; probe_arc278_5a_defrule_query probe; loaded via startup_from_file for the query-only tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

;; A hand-built cold-and-windy rule + a fired session (no defrule needed) — for the query-only tests.

(:wat::core::defn :user::query-coldandwindy-count [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy))))

(:wat::core::defn :user::query-windspeed-count [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired :weather::WindSpeed))))

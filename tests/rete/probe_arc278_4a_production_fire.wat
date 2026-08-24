;; tests/rete/probe_arc278_4a_production_fire.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for production-fire tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

;; Build the fired session for a given WindSpeed location, then flatten production-memory's per-node
;; (PV :- [:wat::core::Record]) values into one `derived` PV (only one rule/ProductionNode exists in each
;; scenario below, so this is exactly the ProductionNode's own derived facts). Two scenario groups:
;; wind at "Oslo" (matches Temperature's loc) and wind at "Bergen" (does not).

(:wat::core::defn :user::pfacts-length-oslo [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     pmem  (:wat::rete::Session/production-memory fired)
     derived (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector]
                  -> :wat::core::PersistentVector
                  (:wat::core::foldl
                    (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record]
                      -> :wat::core::PersistentVector
                      (:wat::core::PersistentVector/conj a f))
                    acc pv))
                (:wat::core::PersistentVector)
                (:wat::core::PersistentMap/values pmem))]
    (:wat::core::length derived)))

(:wat::core::defn :user::fact-type-oslo [] -> :wat::core::String
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     pmem  (:wat::rete::Session/production-memory fired)
     derived (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector]
                  -> :wat::core::PersistentVector
                  (:wat::core::foldl
                    (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record]
                      -> :wat::core::PersistentVector
                      (:wat::core::PersistentVector/conj a f))
                    acc pv))
                (:wat::core::PersistentVector)
                (:wat::core::PersistentMap/values pmem))]
    (:wat::core::type (:wat::core::Option/expect (:wat::core::PersistentVector/get derived 0) "fact"))))

(:wat::core::defn :user::fact-location-oslo [] -> :wat::core::String
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules sess2)
     pmem  (:wat::rete::Session/production-memory fired)
     derived (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector]
                  -> :wat::core::PersistentVector
                  (:wat::core::foldl
                    (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record]
                      -> :wat::core::PersistentVector
                      (:wat::core::PersistentVector/conj a f))
                    acc pv))
                (:wat::core::PersistentVector)
                (:wat::core::PersistentMap/values pmem))]
    (:weather::ColdAndWindy/location (:wat::core::Option/expect (:wat::core::PersistentVector/get derived 0) "fact"))))

(:wat::core::defn :user::pfacts-length-bergen [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     sess2 (:wat::rete::insert sess1 (:weather::WindSpeed :kph 45 :location "Bergen"))
     fired (:wat::rete::fire-rules sess2)
     pmem  (:wat::rete::Session/production-memory fired)
     derived (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector]
                  -> :wat::core::PersistentVector
                  (:wat::core::foldl
                    (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record]
                      -> :wat::core::PersistentVector
                      (:wat::core::PersistentVector/conj a f))
                    acc pv))
                (:wat::core::PersistentVector)
                (:wat::core::PersistentMap/values pmem))]
    (:wat::core::length derived)))

;; HAZARD — one fact per activation, no cross-product. 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc
;; joins → exactly 2 derived facts (NOT 4 from a blind cross, NOT 1 from a clobbered accumulator).
(:wat::core::defn :user::pfacts-length-2x2 [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1 (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::Temperature :celsius 10 :location "Bergen"))
     s3 (:wat::rete::insert s2 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s4 (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     fired (:wat::rete::fire-rules s4)
     pmem  (:wat::rete::Session/production-memory fired)
     derived (:wat::core::foldl
                (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector]
                  -> :wat::core::PersistentVector
                  (:wat::core::foldl
                    (:wat::core::fn [a <- :wat::core::PersistentVector f <- :wat::core::Record]
                      -> :wat::core::PersistentVector
                      (:wat::core::PersistentVector/conj a f))
                    acc pv))
                (:wat::core::PersistentVector)
                (:wat::core::PersistentMap/values pmem))]
    (:wat::core::length derived)))

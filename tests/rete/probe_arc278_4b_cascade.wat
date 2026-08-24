;; tests/rete/probe_arc278_4b_cascade.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for cascade-to-fixpoint tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert [location <- :wat::core::String])

;; A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert (the cascade chain). Gathers ALL
;; derived facts across every ProductionNode (production-memory values flattened into one
;; (PV :- [:wat::core::Record])) as `derived`, then either counts a type or reports the total length.

(:wat::core::defn :user::weatheralert-count-oslo [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
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
    (:wat::core::length
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::filter
          (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool
            (:wat::core::= (:wat::core::type f) "weather::WeatherAlert"))
          derived)))))

(:wat::core::defn :user::coldandwindy-count-oslo [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
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
    (:wat::core::length
      (:wat::core::into (:wat::core::PersistentVector)
        (:wat::core::filter
          (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool
            (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
          derived)))))

(:wat::core::defn :user::derived-length-oslo [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
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

(:wat::core::defn :user::derived-length-bergen [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
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

;; tests/rete/probe_arc278_4c_retraction.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for truth-maintenance tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert [location <- :wat::core::String])

;; The 2-rule chain (reused across all four parts): A: Temp+Wind(same loc)→ColdAndWindy;
;; B: ColdAndWindy→WeatherAlert.

;; ── Part A — the fact-model fix: fire keeps INPUT distinct from DERIVED ─────────────
;; Assert Temp+Wind at Oslo, fire. Session.facts must hold the 2 INPUT facts and NO derived ColdAndWindy.

(:wat::core::defn :user::part-a-temperature-in-facts [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::Temperature"))
      (:wat::rete::Session/facts fired))))))

(:wat::core::defn :user::part-a-coldandwindy-in-facts [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
      (:wat::rete::Session/facts fired))))))

(:wat::core::defn :user::part-a-coldandwindy-derived [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] -> :wat::core::PersistentVector
          (:wat::core::foldl (:wat::core::fn [a <- :wat::core::PersistentVector x <- :wat::core::Record] -> :wat::core::PersistentVector (:wat::core::PersistentVector/conj a x)) acc pv))
        (:wat::core::PersistentVector)
        (:wat::core::PersistentMap/values (:wat::rete::Session/production-memory fired))))))))

;; ── Part B — retraction drops the derived consequence ───────────────────────────

(:wat::core::defn :user::part-b-coldandwindy-derived-after-retract [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0 (:wat::rete::fire-rules s2)
     s3 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s3)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] -> :wat::core::PersistentVector
          (:wat::core::foldl (:wat::core::fn [a <- :wat::core::PersistentVector x <- :wat::core::Record] -> :wat::core::PersistentVector (:wat::core::PersistentVector/conj a x)) acc pv))
        (:wat::core::PersistentVector)
        (:wat::core::PersistentMap/values (:wat::rete::Session/production-memory fired))))))))

;; ── Part C — retraction cascades transitively (CW supported WA) ──────────────────

(:wat::core::defn :user::part-c-weatheralert-derived-after-retract [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0 (:wat::rete::fire-rules s2)
     s3 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s3)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::WeatherAlert"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] -> :wat::core::PersistentVector
          (:wat::core::foldl (:wat::core::fn [a <- :wat::core::PersistentVector x <- :wat::core::Record] -> :wat::core::PersistentVector (:wat::core::PersistentVector/conj a x)) acc pv))
        (:wat::core::PersistentVector)
        (:wat::core::PersistentMap/values (:wat::rete::Session/production-memory fired))))))))

;; ── Part D — retraction is precise: independent derivations survive ──────────────

(:wat::core::defn :user::part-d-coldandwindy-derived-after-retract-oslo [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector ruleA ruleB))
     s1 (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2 (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s3 (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location "Bergen"))
     s4 (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     f0 (:wat::rete::fire-rules s4)
     s5 (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s5)]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::PersistentVector pv <- :wat::core::PersistentVector] -> :wat::core::PersistentVector
          (:wat::core::foldl (:wat::core::fn [a <- :wat::core::PersistentVector x <- :wat::core::Record] -> :wat::core::PersistentVector (:wat::core::PersistentVector/conj a x)) acc pv))
        (:wat::core::PersistentVector)
        (:wat::core::PersistentMap/values (:wat::rete::Session/production-memory fired))))))))

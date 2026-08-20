;; tests/rete/probe_arc278_P4c_native_retraction.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the native retraction differential.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert [location <- :wat::core::String])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


(:wat::rete::defquery :weather::q-WeatherAlert
  :params []
  :when [(?fact <- :weather::WeatherAlert)])


;; A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert (the 4c chain). The fire verb
;; (native fire-rules' vs oracle fire-rules-spec) is 2-valued and every scenario a #[test] needs is
;; a fixed, enumerable named entry — no runtime parameterization.

;; ── single retract: drop a support → its derived ColdAndWindy is gone ──────────────
(:wat::core::defn :user::native-retract-drops-cw [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0    (:wat::rete::fire-rules s2)
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s3)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

(:wat::core::defn :user::oracle-retract-drops-cw [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0    (:wat::rete::fire-rules$oracle s2)
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules$oracle s3)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

;; ── transitive: retract Temp → CW gone → WA (derived from CW) gone too ─────────────
(:wat::core::defn :user::native-retract-cascade-wa [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0    (:wat::rete::fire-rules s2)
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s3)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-WeatherAlert)))))

(:wat::core::defn :user::oracle-retract-cascade-wa [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     f0    (:wat::rete::fire-rules$oracle s2)
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules$oracle s3)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-WeatherAlert)))))

;; ── precise: retract Oslo's Temp; Bergen's independent derivation survives ─────────
(:wat::core::defn :user::native-retract-precise-cw [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s3    (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location "Bergen"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     f0    (:wat::rete::fire-rules s4)
     s5    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s5)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

(:wat::core::defn :user::native-retract-precise-wa [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s3    (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location "Bergen"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     f0    (:wat::rete::fire-rules s4)
     s5    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules s5)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-WeatherAlert)))))

(:wat::core::defn :user::oracle-retract-precise-cw [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s3    (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location "Bergen"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     f0    (:wat::rete::fire-rules$oracle s4)
     s5    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules$oracle s5)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

(:wat::core::defn :user::oracle-retract-precise-wa [] -> :wat::core::i64
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s3    (:wat::rete::insert s2 (:weather::Temperature :celsius 10 :location "Bergen"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     f0    (:wat::rete::fire-rules$oracle s4)
     s5    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:wat::rete::fire-rules$oracle s5)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-WeatherAlert)))))

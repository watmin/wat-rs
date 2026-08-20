;; tests/rete/probe_arc278_P2_native_fire_once.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the native fire-once differential.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(:weather::ColdAndWindy (?location <- :location))])


;; ── staged (not-yet-fired) cold-and-windy scenarios: hand-built rule, Temp(Oslo,15) + Wind(<loc>,45).
;; wind_loc and the fire verb are each 2-valued and every combination a #[test] needs is a fixed,
;; enumerable named entry — no runtime parameterization.

(:wat::core::defn :user::count-native-oslo [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-once s2)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

(:wat::core::defn :user::count-wat-oslo [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-once$oracle s2)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

(:wat::core::defn :user::count-native-bergen [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Bergen"))
     fired (:wat::rete::fire-once s2)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

(:wat::core::defn :user::count-wat-bergen [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Bergen"))
     fired (:wat::rete::fire-once$oracle s2)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

;; native_derives_the_right_fact — the native-derived fact is a ColdAndWindy at "Oslo" (content, not just count).
(:wat::core::defn :user::native-fact-type [] -> :wat::core::String
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-once s2)
     cw    (:wat::core::first
              (:wat::rete::collect-derived
                (:wat::rete::Session/production-memory fired)))]
    (:wat::core::type cw)))

(:wat::core::defn :user::native-fact-location [] -> :wat::core::String
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-once s2)
     cw    (:wat::core::first
              (:wat::rete::collect-derived
                (:wat::rete::Session/production-memory fired)))]
    (:weather::ColdAndWindy/location cw)))

;; native_no_cross_loc_leakage — 2×2: 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc joins → 2 derived.
(:wat::core::defn :user::count-native-2x2 [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::Temperature :celsius 10 :location "Bergen"))
     s3    (:wat::rete::insert s2 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     fired (:wat::rete::fire-once s4)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

(:wat::core::defn :user::count-wat-2x2 [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::core::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::core::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     s0    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert s0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::Temperature :celsius 10 :location "Bergen"))
     s3    (:wat::rete::insert s2 (:weather::WindSpeed :kph 45 :location "Oslo"))
     s4    (:wat::rete::insert s3 (:weather::WindSpeed :kph 50 :location "Bergen"))
     fired (:wat::rete::fire-once$oracle s4)]
    (:wat::core::length (:wat::rete::collect-derived (:wat::rete::Session/production-memory fired)))))

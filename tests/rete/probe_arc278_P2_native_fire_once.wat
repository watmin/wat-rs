;; tests/rete/probe_arc278_P2_native_fire_once.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the native fire-once differential.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

;; fire-once does not re-enter derived ColdAndWindy, so a QueryNode on that type stays empty.
;; The join that the single pass DID populate is the public query mouth.
(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::rete::i64::< ?t 20))
         (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::rete::i64::> ?w 30))])


;; ── staged (not-yet-fired) cold-and-windy scenarios: hand-built rule, Temp(Oslo,15) + Wind(<loc>,45).
;; wind_loc and the fire verb are each 2-valued and every combination a #[test] needs is a fixed,
;; enumerable named entry — no runtime parameterization.

(:wat::core::defn :test::compile-cw [] -> :wat::rete::Session
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::rete::i64::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::rete::i64::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))))

(:wat::core::defn :test::staged-oslo [] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert (:test::compile-cw) (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Oslo")))

(:wat::core::defn :test::staged-bergen [] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert (:test::compile-cw) (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Bergen")))

(:wat::core::defn :test::staged-2x2 [] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert
      (:wat::rete::insert
        (:wat::rete::insert (:test::compile-cw) (:weather::Temperature :celsius 15 :location "Oslo"))
        (:weather::Temperature :celsius 10 :location "Bergen"))
      (:weather::WindSpeed :kph 45 :location "Oslo"))
    (:weather::WindSpeed :kph 50 :location "Bergen")))

(:wat::core::defn :test::cw-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:weather::q-ColdAndWindy))))

(:wat::core::defn :test::cw-loc [s <- :wat::rete::Session] -> :wat::core::String
  (:wat::core::Option/expect
    (:wat::map::get
      (:wat::core::Option/expect
        (:wat::core::PersistentVector/get (:wat::rete::query s (:weather::q-ColdAndWindy)) 0)
        "row")
      "?loc")
    "loc"))

;; rune:vocare(vantage-bypass-test) — fire-once does not re-enter derived facts; the produced ColdAndWindy lives in production-memory, not the query mouth
(:wat::core::defn :test::cw-fact [s <- :wat::rete::Session] -> :weather::ColdAndWindy
  (:wat::core::first (:wat::rete::collect-derived (:wat::rete::Session/production-memory s))))

(:wat::core::defn :user::compile-cw-fires-once-nothing [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once (:test::compile-cw))))

(:wat::core::defn :user::count-native-oslo [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once (:test::staged-oslo))))

(:wat::core::defn :user::count-wat-oslo [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once$oracle (:test::staged-oslo))))

(:wat::core::defn :user::count-native-bergen [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once (:test::staged-bergen))))

(:wat::core::defn :user::count-wat-bergen [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once$oracle (:test::staged-bergen))))

;; native_derives_the_right_fact — the native-derived fact is a ColdAndWindy at "Oslo" (content, not just count).
(:wat::core::defn :user::native-fact-type [] -> :wat::core::String
  (:wat::core::type (:test::cw-fact (:wat::rete::fire-once (:test::staged-oslo)))))

(:wat::core::defn :user::native-fact-location [] -> :wat::core::String
  (:test::cw-loc (:wat::rete::fire-once (:test::staged-oslo))))

;; native_no_cross_loc_leakage — 2×2: 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc joins → 2 derived.
(:wat::core::defn :user::count-native-2x2 [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once (:test::staged-2x2))))

(:wat::core::defn :user::count-wat-2x2 [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-once$oracle (:test::staged-2x2))))

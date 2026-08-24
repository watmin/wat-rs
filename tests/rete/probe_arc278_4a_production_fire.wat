;; tests/rete/probe_arc278_4a_production_fire.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for production-fire tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


;; Wind at "Oslo" (matches Temperature's loc) vs wind at "Bergen" (does not). Harvest is
;; compile-all + fire-rules + query.

(:wat::core::defn :test::compile-cw [] -> :wat::rete::Session
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Oslo")))

(:wat::core::defn :test::seed-bergen [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Bergen")))

(:wat::core::defn :test::seed-2x2 [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert
      (:wat::rete::insert
        (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
        (:weather::Temperature :celsius 10 :location "Bergen"))
      (:weather::WindSpeed :kph 45 :location "Oslo"))
    (:weather::WindSpeed :kph 50 :location "Bergen")))

(:wat::core::defn :test::fired-oslo [] -> :wat::rete::Session
  (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-cw))))

(:wat::core::defn :test::fired-bergen [] -> :wat::rete::Session
  (:wat::rete::fire-rules (:test::seed-bergen (:test::compile-cw))))

(:wat::core::defn :test::fired-2x2 [] -> :wat::rete::Session
  (:wat::rete::fire-rules (:test::seed-2x2 (:test::compile-cw))))

(:wat::core::defn :test::cw-count [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:weather::q-ColdAndWindy))))

(:wat::core::defn :test::cw-fact [s <- :wat::rete::Session] -> :weather::ColdAndWindy
  (:wat::core::Option/expect
    (:wat::core::PersistentMap/get
      (:wat::core::Option/expect
        (:wat::core::PersistentVector/get (:wat::rete::query s (:weather::q-ColdAndWindy)) 0)
        "fact")
      "?fact")
    "fact"))

(:wat::core::defn :user::compile-cw-fires-nothing [] -> :wat::core::i64
  (:test::cw-count (:wat::rete::fire-rules (:test::compile-cw))))

(:wat::core::defn :user::pfacts-length-oslo [] -> :wat::core::i64
  (:test::cw-count (:test::fired-oslo)))

(:wat::core::defn :user::fact-type-oslo [] -> :wat::core::String
  (:wat::core::type (:test::cw-fact (:test::fired-oslo))))

(:wat::core::defn :user::fact-location-oslo [] -> :wat::core::String
  (:weather::ColdAndWindy/location (:test::cw-fact (:test::fired-oslo))))

(:wat::core::defn :user::pfacts-length-bergen [] -> :wat::core::i64
  (:test::cw-count (:test::fired-bergen)))

;; HAZARD — one fact per activation, no cross-product. 2 Temps × 2 Winds / 2 locs → exactly the 2 same-loc
;; joins → exactly 2 derived facts (NOT 4 from a blind cross, NOT 1 from a clobbered accumulator).
(:wat::core::defn :user::pfacts-length-2x2 [] -> :wat::core::i64
  (:test::cw-count (:test::fired-2x2)))

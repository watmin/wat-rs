;; tests/rete/probe_arc278_4b_cascade.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for cascade-to-fixpoint tests.

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


;; A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert (the cascade chain).

(:wat::core::defn :test::compile-ab [] -> :wat::rete::Session
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::rete::core::i64::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::rete::core::i64::> ?w 30)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Oslo")))

(:wat::core::defn :test::seed-bergen [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo"))
    (:weather::WindSpeed :kph 45 :location "Bergen")))

(:wat::core::defn :test::cascade-fired-session [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::cascade-fired-bergen [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules (:test::seed-bergen (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::query-count [s <- :wat::rete::Session q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s q)))

(:wat::core::defn :user::compile-ab-fires-nothing [] -> :wat::core::i64
  (:test::query-count (:wat::core::match (:wat::rete::fire-rules (:test::compile-ab)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy)))

(:wat::core::defn :user::weatheralert-count-oslo [] -> :wat::core::i64
  (:test::query-count (:test::cascade-fired-session) (:weather::q-WeatherAlert)))

(:wat::core::defn :user::coldandwindy-count-oslo [] -> :wat::core::i64
  (:test::query-count (:test::cascade-fired-session) (:weather::q-ColdAndWindy)))

(:wat::core::defn :user::derived-length-oslo [] -> :wat::core::i64
  (:wat::core::let [fired (:test::cascade-fired-session)]
    (:wat::core::i64::+
      (:test::query-count fired (:weather::q-ColdAndWindy))
      (:test::query-count fired (:weather::q-WeatherAlert)))))

(:wat::core::defn :user::derived-length-bergen [] -> :wat::core::i64
  (:wat::core::let [fired (:test::cascade-fired-bergen)]
    (:wat::core::i64::+
      (:test::query-count fired (:weather::q-ColdAndWindy))
      (:test::query-count fired (:weather::q-WeatherAlert)))))

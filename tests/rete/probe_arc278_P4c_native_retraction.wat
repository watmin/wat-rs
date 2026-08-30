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


;; A: Temp+Wind(same loc)→ColdAndWindy; B: ColdAndWindy→WeatherAlert (the 4c chain).
;; Native vs oracle is the one new chapter; compile/seed/retract/count stay named helpers.

(:wat::core::defn :test::compile-ab-rules [] -> :wat::rete::Session
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     ra1   (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "A" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector ra1))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rb1   (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "B" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rb1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:weather::WindSpeed :kph 45 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-bergen [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius 10 :location "Bergen")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:weather::WindSpeed :kph 50 :location "Bergen")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::count-derived [s <- :wat::rete::Session q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s q)))

(:wat::core::defn :test::retract-oslo-temp [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::retract s (:weather::Temperature :celsius 15 :location "Oslo")))

;; ── single retract: drop a support → its derived ColdAndWindy is gone ──────────────
(:wat::core::defn :user::native-retract-drops-cw [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-ab-rules))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::oracle-retract-drops-cw [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-oslo (:test::compile-ab-rules))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules$oracle (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

;; ── transitive: retract Temp → CW gone → WA (derived from CW) gone too ─────────────
(:wat::core::defn :user::native-retract-cascade-wa [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-ab-rules))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-WeatherAlert))))

(:wat::core::defn :user::oracle-retract-cascade-wa [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-oslo (:test::compile-ab-rules))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules$oracle (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-WeatherAlert))))

;; ── precise: retract Oslo's Temp; Bergen's independent derivation survives ─────────
(:wat::core::defn :user::native-retract-precise-cw [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules (:test::seed-bergen (:test::seed-oslo (:test::compile-ab-rules)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::native-retract-precise-wa [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules (:test::seed-bergen (:test::seed-oslo (:test::compile-ab-rules)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-WeatherAlert))))

(:wat::core::defn :user::oracle-retract-precise-cw [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-bergen (:test::seed-oslo (:test::compile-ab-rules)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules$oracle (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::oracle-retract-precise-wa [] -> :wat::core::i64
  (:wat::core::let [f0    (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-bergen (:test::seed-oslo (:test::compile-ab-rules)))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
                   fired (:wat::core::match (:wat::rete::fire-rules$oracle (:test::retract-oslo-temp f0)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:test::count-derived fired (:weather::q-WeatherAlert))))

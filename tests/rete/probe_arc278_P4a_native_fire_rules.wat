;; tests/rete/probe_arc278_P4a_native_fire_rules.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines weather records for the native fire-rules differential.

(:wat::core::defrecord :weather::Temperature  [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed     [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy  [location <- :wat::core::String])
(:wat::core::defrecord :weather::WeatherAlert  [location <- :wat::core::String])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


(:wat::rete::defquery :weather::q-WeatherAlert
  :params []
  :when [(?fact <- :weather::WeatherAlert)])


;; ── layer 0: compile / seed. Each helper has a sibling zero-arg entry. ─────────

(:wat::core::defn :test::compile-cw [] -> :wat::rete::Session
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::rete::core::i64::< ?t 20)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::rete::core::i64::> ?w 30)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))]
    (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))))

(:wat::core::defn :test::compile-ab [] -> :wat::rete::Session
  (:wat::core::let
    [ca1   (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius) (:wat::rete::core::i64::< ?t 20)))
     ca2   (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph) (:wat::rete::core::i64::> ?w 30)))
     rhsA  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     ruleA (:wat::rete::Rule :name "cw" :lhs (:wat::core::PersistentVector ca1 ca2) :rhs (:wat::core::PersistentVector rhsA))
     cb1   (:wat::core::quote (:weather::ColdAndWindy (?loc <- :location)))
     rhsB  (:wat::core::quote (:weather::WeatherAlert ?loc))
     ruleB (:wat::rete::Rule :name "alert" :lhs (:wat::core::PersistentVector cb1) :rhs (:wat::core::PersistentVector rhsB))]
    (:wat::rete::compile-all (:wat::core::PersistentVector ruleA ruleB) (:wat::core::PersistentVector (:weather::q-ColdAndWindy) (:weather::q-WeatherAlert)))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:weather::WindSpeed :kph 45 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-bergen [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius 15 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:weather::WindSpeed :kph 45 :location "Bergen")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::compile-cw-fires-nothing [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::compile-cw)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::compile-ab-fires-nothing [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::compile-ab)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::seed-oslo-session [] -> :wat::rete::Session
  (:test::seed-oslo (:test::compile-cw)))

;; ── single rule: fire-rules (native) on a one-round derivation == fire-rules$oracle ──────────
;; wind_loc and the fire verb are each 2-valued and every combination a #[test] needs is a fixed,
;; enumerable named entry — no runtime parameterization.

(:wat::core::defn :user::single-native-oslo [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-cw))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::single-wat-oslo [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-oslo (:test::compile-cw))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::single-native-bergen [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::seed-bergen (:test::compile-cw))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::single-wat-bergen [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-bergen (:test::compile-cw))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

;; ── cascade: a fact DERIVED by ruleA unlocks ruleB across rounds (THE canary) ─────
;; ruleA: Temperature + WindSpeed (same loc) → ColdAndWindy(loc)
;; ruleB: ColdAndWindy(loc)                  → WeatherAlert(loc)   [fires on a DERIVED fact]

(:wat::core::defn :user::cascade-native-cw [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::cascade-wat-cw [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-oslo (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-ColdAndWindy))))

(:wat::core::defn :user::cascade-native-wa [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:test::seed-oslo (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-WeatherAlert))))

(:wat::core::defn :user::cascade-wat-wa [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:test::seed-oslo (:test::compile-ab))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:weather::q-WeatherAlert))))

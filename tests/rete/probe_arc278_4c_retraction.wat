;; tests/rete/probe_arc278_4c_retraction.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the weather records for truth-maintenance tests.

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


;; The 2-rule chain (reused across all four parts): A: Temp+Wind(same loc)→ColdAndWindy;
;; B: ColdAndWindy→WeatherAlert.

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

(:wat::core::defn :test::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::count-derived [s <- :wat::rete::Session q <- :wat::rete::Query] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s q)))

(:wat::core::defn :user::compile-ab-rules-fires-nothing [] -> :wat::core::i64
  (:test::count-derived (:test::fire (:test::compile-ab-rules)) (:weather::q-ColdAndWindy)))

(:wat::core::defn :user::seed-oslo-then-fire-cw [] -> :wat::core::i64
  (:test::count-derived (:test::fire (:test::seed-oslo (:test::compile-ab-rules))) (:weather::q-ColdAndWindy)))

(:wat::core::defn :user::seed-bergen-then-fire-cw [] -> :wat::core::i64
  (:test::count-derived (:test::fire (:test::seed-bergen (:test::compile-ab-rules))) (:weather::q-ColdAndWindy)))

;; ── Part A — the fact-model fix: fire keeps INPUT distinct from DERIVED ─────────────
;; Assert Temp+Wind at Oslo, fire. Session.facts must hold the 2 INPUT facts and NO derived ColdAndWindy.

(:wat::core::defn :user::part-a-temperature-in-facts [] -> :wat::core::i64
  ;; rune:vocare(vantage-bypass-test) — input-vs-derived layout: Temperature must remain in Session/facts after fire
  (:wat::core::let [fired (:test::fire (:test::seed-oslo (:test::compile-ab-rules)))]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::Temperature"))
      (:wat::rete::Session/facts fired))))))

(:wat::core::defn :user::part-a-coldandwindy-in-facts [] -> :wat::core::i64
  ;; rune:vocare(vantage-bypass-test) — input-vs-derived layout: derived ColdAndWindy must NOT leak into Session/facts
  (:wat::core::let [fired (:test::fire (:test::seed-oslo (:test::compile-ab-rules)))]
    (:wat::core::length (:wat::core::into (:wat::core::PersistentVector) (:wat::core::filter
      (:wat::core::fn [f <- :wat::core::Record] -> :wat::core::bool (:wat::core::= (:wat::core::type f) "weather::ColdAndWindy"))
      (:wat::rete::Session/facts fired))))))

(:wat::core::defn :user::part-a-coldandwindy-derived [] -> :wat::core::i64
  (:test::count-derived (:test::fire (:test::seed-oslo (:test::compile-ab-rules))) (:weather::q-ColdAndWindy)))

;; ── Part B — retraction drops the derived consequence ───────────────────────────

(:wat::core::defn :user::part-b-coldandwindy-derived-after-retract [] -> :wat::core::i64
  (:wat::core::let
    [f0    (:test::fire (:test::seed-oslo (:test::compile-ab-rules)))
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:test::fire s3)]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

;; ── Part C — retraction cascades transitively (CW supported WA) ──────────────────

(:wat::core::defn :user::part-c-weatheralert-derived-after-retract [] -> :wat::core::i64
  (:wat::core::let
    [f0    (:test::fire (:test::seed-oslo (:test::compile-ab-rules)))
     s3    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:test::fire s3)]
    (:test::count-derived fired (:weather::q-WeatherAlert))))

;; ── Part D — retraction is precise: independent derivations survive ──────────────

(:wat::core::defn :user::part-d-coldandwindy-derived-after-retract-oslo [] -> :wat::core::i64
  (:wat::core::let
    [f0    (:test::fire (:test::seed-bergen (:test::seed-oslo (:test::compile-ab-rules))))
     s5    (:wat::rete::retract f0 (:weather::Temperature :celsius 15 :location "Oslo"))
     fired (:test::fire s5)]
    (:test::count-derived fired (:weather::q-ColdAndWindy))))

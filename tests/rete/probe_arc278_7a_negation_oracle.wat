;; tests/rete/probe_arc278_7a_negation_oracle.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the alert::unattended rule for negation oracle tests.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :ops::Maintenance     [location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])

(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))
   (:wat::rete::not (:ops::Maintenance (?loc <- :location)))]
  :then
  [(:alert::Unattended :location ?loc)])

(:wat::rete::defquery :alert::q-Unattended
  :params []
  :when [(?fact <- :alert::Unattended)])


(:wat::core::defn :test::compile-unattended [] -> :wat::rete::Session
  (:wat::rete::compile-all
    (:wat::rete::collect-rules :alert)
    (:wat::core::PersistentVector (:alert::q-Unattended))))

(:wat::core::defn :test::seed-oslo-temp [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius -5 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-maint [s <- :wat::rete::Session loc <- :wat::core::String] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:ops::Maintenance :location loc)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::fire-oracle [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules$oracle s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::count-unattended [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:alert::q-Unattended))))

;; Fire the oracle after the given inserts and count derived Unattended facts.

;; 1 — `:not` PASSES when the negated fact is ABSENT: Temp(Oslo), no Maintenance → 1 Unattended.
(:wat::core::defn :user::unattended-count-absent [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-oslo-temp (:test::compile-unattended)))))

;; 2 — `:not` BLOCKS when the negated fact is PRESENT and MATCHES: Temp(Oslo) + Maintenance(Oslo) → 0.
(:wat::core::defn :user::unattended-count-present-matching [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Oslo"))))

;; 3 — `:not` PASSES when a negated fact exists but at a DIFFERENT binding (the shared-var join-filter):
;; Temp(Oslo) + Maintenance(Bergen) → the Bergen maintenance does NOT match ?loc=Oslo → 1 Unattended.
(:wat::core::defn :user::unattended-count-present-different [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Bergen"))))


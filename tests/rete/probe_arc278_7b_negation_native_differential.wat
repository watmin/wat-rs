;; tests/rete/probe_arc278_7b_negation_native_differential.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the alert::unattended rule for the native/oracle differential.

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
  (:wat::core::match (:wat::rete::compile-all
    (:wat::rete::collect-rules :alert)
    (:wat::core::PersistentVector (:alert::q-Unattended))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")]))

(:wat::core::defn :test::seed-oslo-temp [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius -5 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(:wat::core::defn :test::seed-maint [s <- :wat::rete::Session loc <- :wat::core::String] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s (:ops::Maintenance :location loc)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")]))

(:wat::core::defn :test::fire-native [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))

(:wat::core::defn :test::fire-oracle [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules$oracle s) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))

(:wat::core::defn :test::count-unattended [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:alert::q-Unattended))))

;; Fire via `fire` after the given inserts; count derived Unattended facts. Six combos:
;; {native, oracle} x {absent, present-matching, present-different}.

(:wat::core::defn :user::native-absent [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-native
      (:test::seed-oslo-temp (:test::compile-unattended)))))

(:wat::core::defn :user::oracle-absent [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-oslo-temp (:test::compile-unattended)))))

(:wat::core::defn :user::native-present-matching [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-native
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Oslo"))))

(:wat::core::defn :user::oracle-present-matching [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Oslo"))))

(:wat::core::defn :user::native-present-different [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-native
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Bergen"))))

(:wat::core::defn :user::oracle-present-different [] -> :wat::core::i64
  (:test::count-unattended
    (:test::fire-oracle
      (:test::seed-maint
        (:test::seed-oslo-temp (:test::compile-unattended))
        "Bergen"))))


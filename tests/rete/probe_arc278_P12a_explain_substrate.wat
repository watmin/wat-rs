;; tests/rete/probe_arc278_P12a_explain_substrate.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two-level weather cascade for explain-substrate tests.

(:wat::core::defrecord :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])
(:wat::core::defrecord :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::core::i64::< ?c 0))
   (:weather::WindSpeed   (?loc <- :location) (?k <- :kph)     (:wat::rete::core::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy ?c ?k)])

(:wat::rete::defrule :weather::alert
  :when
  [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]
  :then
  [(:weather::WeatherAlert :celsius ?c :kph ?k)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


(:wat::core::defn :test::compile-weather [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all
    (:wat::rete::collect-rules :weather)
    (:wat::core::PersistentVector (:weather::q-ColdAndWindy))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::seed-oslo [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:weather::Temperature :celsius -5 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:weather::WindSpeed :kph 40 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::explain-oslo [] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain (:test::seed-oslo (:test::compile-weather))) ((:wat::rete::FireOutcome::Fired __explained) __explained) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules-explain: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules-explain: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::explain-oslo-oracle [] -> :wat::rete::Explained
  (:wat::core::match (:wat::rete::fire-rules-explain$oracle (:test::seed-oslo (:test::compile-weather))) ((:wat::rete::FireOutcome::Fired __explained) __explained) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules-explain: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules-explain: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::compile-weather-fires-nothing [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules (:test::compile-weather)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
      (:weather::q-ColdAndWindy))))

;; 1. CLOSURE FIDELITY — explain mode derives the same facts as the fast path: `Explained/session` is a real
;; fired session, and the ColdAndWindy closure count is 1 (diagnostics add provenance, never change WHAT fires).
(:wat::core::defn :user::closure-fidelity-coldandwindy-count [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query
      (:wat::rete::Explained/session (:test::explain-oslo))
      (:weather::q-ColdAndWindy))))

;; 2. INDEX POPULATED — the support map has one entry per derived fact: ColdAndWindy + WeatherAlert = 2.
(:wat::core::defn :user::support-index-length [] -> :wat::core::i64
  (:wat::core::PersistentMap/length (:wat::rete::Explained/support (:test::explain-oslo))))

;; 3. CHAINS CAPTURED — each entry's producing token carries its real `matches` support chain. Sum of chain
;; lengths over all support entries: ColdAndWindy's token has 2 edges (Temperature, WindSpeed), WeatherAlert's
;; has 1 (ColdAndWindy) → 3. This proves the index stores the real provenance, not just fact keys.
(:wat::core::defn :user::support-chains-total-length [] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  sv <- :wat::rete::Support]
      -> :wat::core::i64
      (:wat::core::i64::+ acc
        (:wat::core::length (:wat::rete::Token/matches (:wat::rete::Support/token sv)))))
    0
    (:wat::core::PersistentMap/values (:wat::rete::Explained/support (:test::explain-oslo)))))

;; 4. ORACLE SIGIL — fire-rules-explain$oracle matches native support cardinality.
(:wat::core::defn :user::support-index-length-oracle [] -> :wat::core::i64
  (:wat::core::PersistentMap/length (:wat::rete::Explained/support (:test::explain-oslo-oracle))))

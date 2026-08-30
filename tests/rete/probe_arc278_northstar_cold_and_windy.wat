;; tests/rete/probe_arc278_northstar_cold_and_windy.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). The north-star defrule: cold-and-windy end-to-end DSL spec.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature
     (?loc <- :location)
     (?c   <- :celsius)
     (:wat::rete::core::i64::< ?c 20))
   (:weather::WindSpeed
     (?loc <- :location)
     (?k   <- :kph)
     (:wat::rete::core::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy :location ?loc)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


;; The lifecycle, value-threaded: collect → compile → insert → insert → fire → query, then COUNT the
;; derived facts (wrapped in `length` so the Rust driver just-evals to a scalar).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules    (:wat::rete::collect-rules :weather)
       session  (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:weather::q-ColdAndWindy))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
       session  (:wat::core::match (:wat::rete::insert session (:weather::Temperature :celsius 15 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
       session  (:wat::core::match (:wat::rete::insert session (:weather::WindSpeed    :kph 45 :location "Oslo")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
       fired    (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
      (:wat::rete::query fired (:weather::q-ColdAndWindy)))))


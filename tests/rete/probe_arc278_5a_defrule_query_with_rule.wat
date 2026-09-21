;; tests/rete/probe_arc278_5a_defrule_query_with_rule.wat — records + defrule fixture for the
;; probe_arc278_5a_defrule_query probe; loaded via startup_from_file for the defrule tests.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc :- :location) (?c :- :celsius) (:wat::rete::i64::< ?c 20))
   (:weather::WindSpeed    (?loc :- :location) (?k :- :kph)     (:wat::rete::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy :location ?loc)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact :- :weather::ColdAndWindy)])


;; Calling the generated zero-arg fn yields a Rule with the expected name + lhs/rhs arity.
(:wat::core::defn :user::rule-name [] -> :wat::core::String
  (:wat::rete::Rule/name (:weather::cold-and-windy)))

(:wat::core::defn :user::rule-lhs-length [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::Rule/lhs (:weather::cold-and-windy))))

(:wat::core::defn :user::rule-rhs-length [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::Rule/rhs (:weather::cold-and-windy))))

;; Collect the one rule MANUALLY (call its fn), compile, insert, fire, query → one ColdAndWindy.
(:wat::core::defn :user::defrule-fires-end-to-end [] -> :wat::core::i64
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:weather::cold-and-windy))
     sess0 (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:weather::q-ColdAndWindy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     s1    (:wat::core::match (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     s2    (:wat::core::match (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (:wat::core::match (:wat::rete::fire-rules s2) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

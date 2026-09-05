;; VIGILIA experiri probe — insert STAGES one; retract REMOVES every equal fact.
;;
;; `:wat::rete::retract` (wat/rete/oracle/insert.wat:100) rebuilds Session.facts with a
;; foldl that keeps `f` only when `(not (= f fact))`. Two structurally-equal staged copies
;; are therefore BOTH dropped by one retract. `insert` stages exactly one per call.

(:wat::core::defrecord :vrm::F [k <- :wat::core::i64])
(:wat::core::defrecord :vrm::G [k <- :wat::core::i64])
(:wat::core::defrecord :vrm::Seen [k <- :wat::core::i64])

(:wat::rete::defrule :vrm::mark
  :when [(:vrm::F (?k <- :k)) (:vrm::G (?k <- :k))]
  :then [(:vrm::Seen :k ?k)])

(:wat::rete::defquery :vrm::q-seen :params [] :when [(?f <- :vrm::Seen)])

(:wat::core::defn :vrm::base [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :vrm)
    (:wat::core::PersistentVector (:vrm::q-seen)))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
      (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

(:wat::core::defn :vrm::ins [s <- :wat::rete::Session f <- :wat::core::Record] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert s f)
    ((:wat::rete::InsertOutcome::Inserted __staged) __staged)
    ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count)
      (:wat::kernel::assertion-failed! "insert: ceiling" :wat::core::None :wat::core::None))))

(:wat::core::defn :vrm::fire [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules s)
    ((:wat::rete::FireOutcome::Fired __fired) __fired)
    ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds)
      (:wat::kernel::assertion-failed! "fire-rules: ceiling" :wat::core::None :wat::core::None))
    ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still)
      (:wat::kernel::assertion-failed! "fire-rules: cap" :wat::core::None :wat::core::None))))

;; [facts after 2 identical inserts,
;;  facts after ONE retract of that fact,
;;  Seen rows after that retract + refire,
;;  facts after 2 identical inserts and NO retract (control)]
(:wat::core::defn :user::multiplicity [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let
    [s2   (:vrm::ins (:vrm::ins (:vrm::ins (:vrm::base) (:vrm::G :k 1)) (:vrm::F :k 1)) (:vrm::F :k 1))
     f0   (:vrm::fire s2)
     r1   (:wat::rete::retract f0 (:vrm::F :k 1))
     f1   (:vrm::fire r1)]
    (:wat::core::mapv
      (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
      (:wat::core::PersistentVector
        (:wat::core::length (:wat::rete::Session/facts f0))
        (:wat::core::length (:wat::rete::Session/facts r1))
        (:wat::core::length (:wat::rete::query f1 (:vrm::q-seen)))
        (:wat::core::length (:wat::rete::query f0 (:vrm::q-seen)))))))

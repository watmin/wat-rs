;; tests/rete/probe_arc278_alpha_is_fire_scoped.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()).
;;
;; Mirrors probe_arc278_2b_insert_alpha.wat's smallest alpha-populating workload (:user::Temp +
;; `(> ?t 20)`), extended with a non-empty RHS (2b's rule had an empty :rhs, deriving nothing) so a
;; derived-fact differential exists alongside the alpha-key-count differential.

(:wat::core::defrecord :afs::Temp [value <- :wat::core::i64])
(:wat::core::defrecord :afs::Hot  [value <- :wat::core::i64])

(:wat::rete::defquery :afs::q-Hot
  :params []
  :when [(?fact <- :afs::Hot)])


;; One condition, one matching fact (25) and one non-matching fact (15, fails > 20); RHS derives
;; :afs::Hot from the matching fact only.
(:wat::core::defn :afs::built [] -> :wat::rete::Session
  (:wat::core::let
    [cond  (:wat::core::quote (:afs::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20)))
     rhs1  (:wat::core::quote (:afs::Hot ?t))
     rule  (:wat::rete::Rule :name "afs" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:afs::q-Hot))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
     sess1 (:wat::core::match (:wat::rete::insert sess0 (:afs::Temp :value 25)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
     sess2 (:wat::core::match (:wat::rete::insert sess1 (:afs::Temp :value 15)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))]
    sess2))

;; (1) native-alpha-key-count — fired via native fixpoint `fire-rules`. Expect 0: the clear happened.
(:wat::core::defn :user::native-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::core::match (:wat::rete::fire-rules (:afs::built)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     ;; rune:vocare(vantage-bypass-test) — fire-scoped alpha is implementer layout, not query
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (2) oracle-alpha-key-count — fired via `fire-rules$oracle` (the wat ORACLE, never optimized). Expect
;; 0: `fire-stratified` returns alpha-memory empty (wat/rete/oracle/fire.wat:349) — asserted here, not assumed.
(:wat::core::defn :user::oracle-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::core::match (:wat::rete::fire-rules$oracle (:afs::built)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     ;; rune:vocare(vantage-bypass-test) — fire-scoped alpha is implementer layout, not query
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (4) single-pass-alpha-key-count — fired via native `fire-once` (single-pass). Expect > 0: THE
;; ANCHOR — proves this workload really does populate alpha, so (1)/(2)/(3) are not vacuously true
;; over a workload that matches nothing. `fire-once` is deliberately left untouched by this stone.
(:wat::core::defn :user::single-pass-alpha-key-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:wat::core::match (:wat::rete::fire-once (:afs::built)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-once: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-once: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
     ;; rune:vocare(vantage-bypass-test) — fire-scoped alpha is implementer layout, not query
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (5) native-derived-count / oracle-derived-count — the RESULT (production output), expected equal
;; and > 0: closing the alpha divergence must not move what fire actually derives.
(:wat::core::defn :user::native-derived-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules (:afs::built)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:afs::q-Hot))))

(:wat::core::defn :user::oracle-derived-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query (:wat::core::match (:wat::rete::fire-rules$oracle (:afs::built)) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:afs::q-Hot))))

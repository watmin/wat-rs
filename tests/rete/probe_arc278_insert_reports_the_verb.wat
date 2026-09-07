;; Co-located fixture for probe_arc278_insert_reports_the_verb.rs.
;; Runtime TypeMismatch on a non-Record fact — not a .wat.bad; this loads and type-checks.

(:wat::core::defrecord :l22::F [n <- :wat::core::i64])

(:wat::core::defn :user::empty-session [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::compile (:wat::core::PersistentVector))
    ((:wat::rete::CompileOutcome::Compiled __session) __session)
    ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type)
     (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None))))

;; 3+-arity `insert`: session, one Record, one non-Record. The user wrote `insert`.
(:wat::core::defn :user::insert-non-record [] -> :wat::rete::InsertOutcome
  (:wat::rete::insert (:user::empty-session) (:l22::F 1) (:wat::core::i64::+ 1 1)))

;; `insert-all`: session + a PersistentVector of a non-Record. A typed
;; `insert-all` call of that vector is a check-time refusal (facts are
;; `(PersistentVector :- [Record])`). `apply` spreads at runtime so
;; `require_record_fact` is the one that fires. The user wrote `insert-all`.
(:wat::core::defn :user::insert-all-non-record [] -> :wat::rete::InsertOutcome
  (:wat::core::apply
    :wat::rete::insert-all
    (:user::empty-session)
    [(:wat::core::PersistentVector (:wat::core::i64::+ 1 1))]))

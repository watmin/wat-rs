;; Scratch — CAN a rete RHS construct an enum-variant value into a fact field?
;;
;; wat/fmt.wat:9 asserts it cannot: "kind is a String, not a keyword: rete RHS may
;; insert a string literal but refuses a keyword literal (RhsUnresolvableOperand)."
;; That is TRUE of a bare keyword — src/rete/validate.rs:1060 reads `:name` in a RHS
;; as a FIELD REFERENCE, so `:block` is ambiguous with a field named `block`.
;;
;; But an enum VARIANT is a call form, not a keyword. validate.rs:1132 records that
;; arc 278 Stone B REMOVED List from the never-resolves set, and validate.rs:1166
;; describes a recursive walk for "a nested aggregate or enum-variant constructor".
;; So the blocker's premise may have expired. `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`
;;
;; This probe fails at exactly one place if it is still refused, with everything
;; around it clean.

(:wat::core::defenum :user::BreakKind :wat::enum::Pure
  :Block []
  :Align [])

(:wat::core::defrecord :user::Seed
  [id <- :wat::core::i64])

(:wat::core::defrecord :user::Broken
  [id   <- :wat::core::i64
   kind <- :user::BreakKind])

;; THE QUESTION: an enum-variant constructor in a :then value position.
(:wat::rete::defrule :user::enum-in-a-rhs
  :when [(:user::Seed (?i :- :id))]
  :then [(:user::Broken :id ?i :kind (:user::BreakKind.Block {}))])

(:wat::rete::defquery :user::q-Broken
  :params []
  :when [(?fact :- :user::Broken)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules    (:wat::rete::collect-rules :user)
     template (:wat::core::match (:wat::rete::compile-all rules
                (:wat::core::PersistentVector (:user::q-Broken))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
     fired    (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::insert template (:user::Seed :id 7)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     out      (:wat::rete::query fired (:user::q-Broken))]
    (:wat::kernel::println
      (:wat::string::interpolate "RULES={r} BROKEN-FACTS={b}"
        :r (:wat::i64::to-string (:wat::core::length rules))
        :b (:wat::i64::to-string (:wat::core::length out))))))

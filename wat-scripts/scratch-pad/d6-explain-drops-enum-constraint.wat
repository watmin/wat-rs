;; d6-explain-drops-enum-constraint.wat — orchestrator reconnaissance for work-list D6.
;;
;; QUESTION: `step_payload.rs`'s doc promises "constraints: the rule's satisfied predicates with
;; bound values substituted", under a bolded "Faithfulness by construction". Three `continue`s can
;; drop a constraint silently; the sharpest is `value_to_ast_literal` (matcher.rs:979), whose arms
;; are bool / f64 / i64 / String / Unit / keyword — with NO `Value::Enum`.
;;
;; So: does an ENUM-operand constraint reach the explain payload, or vanish?
;; The i64 constraint on the SAME condition is the CONTROL — if neither appears the probe is
;; broken, not the engine. Expect 2 constraints; a print of 1 is the defect.

(:wat::core::defenum :d6::Grade :wat::enum::Pure :Hi :Lo)

(:wat::core::defrecord :d6::Reading [n <- :wat::core::i64  grade <- :d6::Grade])
(:wat::core::defrecord :d6::Hit     [n <- :wat::core::i64])

(:wat::rete::defrule :d6::hit
  :when
  [(:d6::Reading (?n <- :n) (?g <- :grade)
                 (:wat::rete::i64::> ?n 5)
                 (:wat::rete::core::enum::= ?g :d6::Grade.Hi))]
  :then
  [(:d6::Hit :n ?n)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :d6)
     session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __s} __s] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f} (:wat::kernel::assertion-failed! :message "may not terminate")])
     session (:wat::core::match (:wat::rete::insert session (:d6::Reading :n 9 :grade :d6::Grade.Hi)) [:wat::rete::InsertOutcome.Inserted {:session __st} __st] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __l :used __u :staged __c} (:wat::kernel::assertion-failed! :message "ceiling")])
     fired   (:wat::core::match (:wat::rete::fire-rules-explain session) [:wat::rete::FireOutcome.Fired {:value __e} __e] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r} (:wat::kernel::assertion-failed! :message "ceiling")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s} (:wat::kernel::assertion-failed! :message "roundcap")])
     node    (:wat::rete::explain fired (:d6::Hit :n 9))
     steps   (:wat::rete::DerivationNode/via node)]
    (:wat::kernel::println
      (:wat::rete::DerivationStep/constraints (:wat::core::first steps)))))

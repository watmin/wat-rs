;; probe_arc278_D6_constraint_omission_unit.wat — THE ★ ARM of work-list D6.
;;
;; One condition, two constraints. `i64::>` is the CONTROL — it rendered before the strike and must
;; render identically after, so a payload that lost it means the probe drifted, not that the engine
;; improved. `enum::=` against the unit variant `:d6u::Grade::Hi` is the SUBJECT.
;;
;; AT HEAD (c9bb8044b) this printed ONE constraint:
;;     #wat.core/PersistentVector [(:wat.rete.core.i64/> 9 5)]
;; The enum clause was dropped with no diagnostic, behind TWO stacked gates:
;;   1. `eval_step_payload` passed `sym: None` to `resolve_operand`, so the RIGHT operand — the
;;      unit-variant keyword — resolved to nothing (an enum variant needs the SymbolTable).
;;   2. `value_to_ast_literal` had no `Value::Enum` arm, so even a resolved variant had no
;;      spelling. Curing only (1) moves the drop one line down and changes nothing here.
;; Both must land for this fixture to print two constraints, which is why it is the row that
;; separates a two-gate fix from a one-gate one.

(:wat::core::defenum :d6u::Grade :wat::enum::Pure :Hi :Lo)

(:wat::core::defrecord :d6u::Reading [n <- :wat::core::i64  grade <- :d6u::Grade])
(:wat::core::defrecord :d6u::Hit     [n <- :wat::core::i64])

(:wat::rete::defrule :d6u::hit
  :when
  [(:d6u::Reading (?n <- :n) (?g <- :grade)
                  (:wat::rete::core::i64::> ?n 5)
                  (:wat::rete::core::enum::= ?g :d6u::Grade::Hi))]
  :then
  [(:d6u::Hit :n ?n)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :d6u)
     session (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __s) __s) ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:d6u::Reading :n 9 :grade :d6u::Grade::Hi)) ((:wat::rete::InsertOutcome::Inserted __st) __st) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules-explain session) ((:wat::rete::FireOutcome::Fired __e) __e) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None)))
     node    (:wat::rete::explain fired (:d6u::Hit :n 9))
     steps   (:wat::rete::DerivationNode/via node)]
    (:wat::kernel::println
      (:wat::rete::DerivationStep/constraints (:wat::core::first steps)))))

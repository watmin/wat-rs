;; probe_arc278_D6_constraint_omission_tagged.wat — THE RESIDUE ARM of work-list D6.
;;
;; The strike gave `value_to_ast_literal` an enum arm for UNIT variants only. A TAGGED variant is
;; deliberately still unrenderable: it is never a literal the author wrote (it can only arrive
;; bound from a fact field), `(:E::V 1)` and `#E/V [1]` are both defensible spellings with nothing
;; to choose between them, and either would need every field value recursively re-encoded.
;;
;; What this fixture pins is therefore NOT that it renders — it is that it no longer VANISHES.
;; `(:wat::rete::core::enum::not= ?g :d6t::Grade::Rejected)` is satisfied by a `:Scored 7` grade,
;; so the rule fires; the payload keeps the constraint's POSITION with the omission marker
;; `(:wat::rete::explain::constraint-not-rendered …)` naming the op, the operand and the reason.
;;
;; The `i64::>` clause on the same condition is the CONTROL: it must render normally, so a payload
;; of length 2 with one real form and one marker is the whole verdict.

(:wat::core::defenum :d6t::Grade :wat::enum::Pure
  :Rejected []
  :Scored   [points <- :wat::core::i64])

(:wat::core::defrecord :d6t::Reading [n <- :wat::core::i64  grade <- :d6t::Grade])
(:wat::core::defrecord :d6t::Hit     [n <- :wat::core::i64])

(:wat::rete::defrule :d6t::hit
  :when
  [(:d6t::Reading (?n <- :n) (?g <- :grade)
                  (:wat::rete::core::i64::> ?n 5)
                  (:wat::rete::core::enum::not= ?g :d6t::Grade::Rejected))]
  :then
  [(:d6t::Hit :n ?n)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :d6t)
     session (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __s) __s) ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::assertion-failed! "may not terminate" :wat::core::None :wat::core::None)))
     session (:wat::core::match (:wat::rete::insert session (:d6t::Reading :n 9 :grade (:d6t::Grade::Scored 7))) ((:wat::rete::InsertOutcome::Inserted __st) __st) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __l __u __c) (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None)))
     fired   (:wat::core::match (:wat::rete::fire-rules-explain session) ((:wat::rete::FireOutcome::Fired __e) __e) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __l __u __r) (:wat::kernel::assertion-failed! "ceiling" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __c __s) (:wat::kernel::assertion-failed! "roundcap" :wat::core::None :wat::core::None)))
     node    (:wat::rete::explain fired (:d6t::Hit :n 9))
     steps   (:wat::rete::DerivationNode/via node)]
    (:wat::kernel::println
      (:wat::rete::DerivationStep/constraints (:wat::core::first steps)))))

;; Fixture BESIDE probe_arc278_accumulate_from_types.rs — THE OVER-REJECTION CONTROL.
;;
;; ⛔ LOAD-BEARING, and it is the half most likely to be skipped. The cure under test makes an
;; `accumulate` condition's `:from` inner REFUSE an ill-typed predicate for the first time, and the
;; failure that scores full marks on every refusal row is a cure that refuses EVERYTHING. This file
;; is the half that proves legal `accumulate` rules still compile — mirrors
;; `probe_arc278_fence_interior_types.wat`, same shape, same reason.
;;
;; D10 shipped exactly this trap on the `:then` side and its header names the cure: `OperandType`
;; separates knowable-and-WRONG from NOT-KNOWABLE, and only the middle one is a refusal.
;; `check_constraint_types` already carries `UnboundInThisRule` and `ComputedNotDerivableHere` for
;; that reason. Row 4 below is the row that makes a cure ignoring that distinction go red here
;; instead of shipping and breaking the corpus.
;;
;; Four rows, derived from the acc-form vocabulary and the spec, not from usage — `accumulate` has
;; ZERO uses in the `.wat` corpus (see DESIGN.md), so no corpus-driven method could have built this.

(:wat::core::defrecord :tac::Station   [location <- :wat::core::String])
(:wat::core::defrecord :tac::Reading   [location <- :wat::core::String  value <- :wat::core::i64])
(:wat::core::defrecord :tac::Threshold [min <- :wat::core::i64])

;; row 1 — a PLAIN BIND, no clauses beyond it. The shape the `:from` arm already handled.
(:wat::rete::defrule :tac::plain-bind
  :when [(:tac::Station (?loc <- :location))
         (?n <- (:wat::rete::acc::count) :from (:tac::Reading (?loc <- :location)))]
  :then [])

;; row 2 — a WELL-TYPED inline constraint inside `:from`'s inner. `?v` is bound to `:value` (i64);
;; `i64::>` is the matching comparator. This is exactly the shape the widened check must accept.
(:wat::rete::defrule :tac::typed-constraint
  :when [(:tac::Station (?loc <- :location))
         (?n <- (:wat::rete::acc::count)
             :from (:tac::Reading (?loc <- :location) (?v <- :value)
                     (:wat::rete::core::i64::> ?v 0)))]
  :then [])

;; row 3 — a JOIN VARIABLE bound in an EARLIER condition (`?min`, from `:tac::Threshold`), read
;; FREELY (not `<-`) inside `:from`'s inner constraint. The free `?var` stays free — cross-condition
;; join — and only the `:field` side is schema-checked (DESIGN-rete-defrule-wall.md). A cure that
;; cannot resolve an earlier bind from inside the recursed `:from` walk would wrongly refuse this.
(:wat::rete::defrule :tac::earlier-join
  :when [(:tac::Station (?loc <- :location))
         (:tac::Threshold (?min <- :min))
         (?n <- (:wat::rete::acc::count)
             :from (:tac::Reading (?loc <- :location) (?v <- :value)
                     (:wat::rete::core::i64::> ?v ?min)))]
  :then [])

;; row 4 — ⭐ NOT KNOWABLE. The right operand is a `cond` — a `Form`-class rete op whose vocabulary
;; row carries NO `TypeScheme` (`vocabulary.rs`: `Form` rows state no `ret`), so
;; `resolve_operand_type` returns `OperandType::ComputedNotDerivableHere` rather than a type — the
;; SAME construction `probe_arc278_D10_then_field_types_notknowable.wat`'s `nk1` uses on the
;; `:then` side. ⛔ **NOT** the fence's `i64::+ ?v 1 :undefined 0` trick: that op's vocabulary row
;; declares `ret: Ret::Is(ParamType::I64)` (a REAL, knowable return type — `Fallback`-class, still
;; total) — driven here (both a live and a MUTATED run) to confirm it resolves as `Resolved("i64")`
;; and never reaches `ComputedNotDerivableHere` at all, so it does not exercise this guard. A cure
;; that refuses every operand it cannot type passes rows 1-3 and both `.wat.bad` siblings, and
;; still stops legal rules from compiling. This row is the only thing here that catches it.
;; (Compile-only, like its fence ancestor — the runtime behavior of `cond` inside a `:from` filter
;; is not this fixture's claim.)
(:wat::rete::defrule :tac::not-knowable
  :when [(:tac::Station (?loc <- :location))
         (?n <- (:wat::rete::acc::count)
             :from (:tac::Reading (?loc <- :location) (?v <- :value)
                     (:wat::rete::core::i64::= ?v
                       (:wat::rete::core::cond
                         ((:wat::rete::core::string::= ?loc "Oslo") 10)
                         (:else 999)))))]
  :then [])

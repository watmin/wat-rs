;; Fixture BESIDE probe_arc278_fence_interior_types.rs — THE OVER-REJECTION CONTROL.
;;
;; ⛔ LOAD-BEARING, and it is the half most likely to be skipped. The cure under test makes a
;; `where` fence's interior REFUSE for the first time, and the failure that scores full marks on
;; every refusal row is a cure that refuses EVERYTHING. The two `.wat.bad` siblings prove the
;; refusals; this file is the half that proves legal fences still compile.
;;
;; D10 shipped exactly this trap on the `:then` side and its header names the cure:
;; `OperandType` separates knowable-and-WRONG from NOT-KNOWABLE, and only the middle one is a
;; refusal. `check_constraint_types` already carries `UnboundInThisRule` and
;; `ComputedNotDerivableHere` for that reason. Row 3 below is the row that makes a cure ignoring
;; the distinction go red here instead of shipping and breaking the corpus.
;;
;; ⛔ EVERY FENCE HERE READS BOTH CONDITIONS, on purpose. A fence needing only one condition's
;; bindings is the hoistable join-blowup form the corpus was migrated away from at `feb5fae91`;
;; a fixture in a teaching directory must not re-teach it.

(:wat::core::defrecord :tgc::N [k <- :wat::core::i64  s <- :wat::core::String])

;; row 1 — knowable and RIGHT: i64 comparator over two i64-bound join vars.
(:wat::rete::defrule :tgc::typed-i64
  :when [(:tgc::N (?k <- :k))
         (:tgc::N (?j <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k ?j))]
  :then [])

;; row 2 — knowable and RIGHT at a DIFFERENT type. A cure that hardcodes one type passes row 1.
(:wat::rete::defrule :tgc::typed-string
  :when [(:tgc::N (?s <- :s))
         (:tgc::N (?t <- :s))
         (:wat::rete::where (:wat::rete::core::string::= ?s ?t))]
  :then [])

;; row 3 — ⭐ NOT KNOWABLE. The right operand is a computed rete form, not a bare bind. A cure
;; that refuses every operand it cannot type passes rows 1-2 and both `.wat.bad` siblings, and
;; still stops legal rules from compiling. This row is the only thing that catches it.
(:wat::rete::defrule :tgc::computed-operand
  :when [(:tgc::N (?k <- :k))
         (:tgc::N (?j <- :k))
         (:wat::rete::where (:wat::rete::core::i64::= ?k (:wat::rete::core::i64::+ ?j 1 :undefined 0)))]
  :then [])

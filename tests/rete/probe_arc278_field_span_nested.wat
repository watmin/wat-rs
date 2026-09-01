;; strike-field-span ROW 2 — DISCONFIRMING. The NESTED-CONSTRUCTOR producer is NOT REACHABLE,
;; and this fixture is the drive that says so.
;;
;; `walk_nested_constructors`'s aggregate branch (`validate/mod.rs`) opens with
;; `if let Some(TypeDef::Aggregate(_)) = types.get(head)` — it looks for the record type as the
;; HEAD of the nested form. By the time the freeze-time wall runs, `defrecord`'s companion macro
;; has already lowered EVERY record-constructor call form to the post-lowering spelling, where the
;; type is ARGUMENT 0:
;;
;;     (:fsn::Inner :nope ?k)   ->   (:wat::core::kwargs-construct :fsn::Inner :nope ?k)
;;
;; Driven with an `eprintln!` inside that branch: the head it actually receives is
;; `":wat::core::kwargs-construct"` and `types.get` on it is `None`, for the kwargs spelling, the
;; single-arg positional spelling, the multi-arg positional spelling, and with the OUTER item
;; written positionally. The walk then falls through to its generic recursion, where the record
;; keyword is a bare `Keyword` (not a `List`) and nothing is checked.
;;
;; So the aggregate branch's FOUR findings — `UnknownField`, `RhsMissingFields`,
;; `RhsArityMismatch`, `RhsPositionalConstructionRetired` — are all unreachable for a nested
;; record constructor. Its sibling enum-variant branch IS live (an enum variant is not lowered),
;; which is why the walk as a whole looks exercised.
;;
;; ⛔ THIS IS NOT THIS STRIKE'S CUT. `purity.rs` hit the identical class and was TAUGHT the
;; post-lowering shape (*"the door simply matched the pre-lowering spelling, where the type is the
;; HEAD, and could not see the post-lowering spelling, where the type is ARGUMENT 0"*).
;; `walk_nested_constructors` never was. Teaching it is a wall-reachability strike touching four
;; error kinds, not a span strike. This fixture PINS the gap so it is a red test the day someone
;; takes it, rather than a paragraph in a stone.
;;
;; The nested constructor below names a field `:fsn::Inner` does not declare AND under-supplies
;; the one it does. Both are freeze-time refusals if the wall ever sees this form. Today the
;; program compiles and `main` runs.

(:wat::core::defrecord :fsn::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :fsn::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :fsn::Outer [k <- :wat::core::i64  inner <- :fsn::Inner])

(:wat::rete::defrule :fsn::r
  :when [(:fsn::Src (?k <- :k))]
  :then [(:fsn::Outer :k ?k :inner (:fsn::Inner :nope ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ACCEPTED-UNVALIDATED"))

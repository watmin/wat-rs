;; strike-field-span ROW 2 / strike-nested-wall — the NESTED-CONSTRUCTOR path to `UnknownField`.
;;
;; ⚠ THIS FIXTURE'S VERDICT WAS FLIPPED, AND ITS SOURCE WAS NOT TOUCHED. It was written as a
;; DISCONFIRMING pin: the program below compiled and ran, printing `ACCEPTED-UNVALIDATED`, because
;; `walk_nested_constructors` could not see the form at all. It is now a freeze REFUSAL, and the
;; same bytes prove the opposite thing. Nothing about the program changed; the wall did.
;;
;; The mechanism was ORPHANING. `walk_nested_constructors` looked for the record type as the HEAD
;; of the nested form, but by freeze time `defrecord`'s companion macro has already lowered EVERY
;; record-constructor call to the post-lowering spelling, where the type is ARGUMENT 0:
;;
;;     (:fsn::Inner :nope ?k)   ->   (:wat::core::kwargs-construct :fsn::Inner :nope ?k)
;;
;; So `types.get(head)` was `None` — for the kwargs spelling, the single-arg positional spelling,
;; the multi-arg positional spelling, and with the OUTER item written positionally — and the
;; aggregate branch's FOUR findings (`UnknownField`, `RhsMissingFields`, `RhsArityMismatch`,
;; `RhsPositionalConstructionRetired`) were all unreachable. Its sibling enum-variant branch WAS
;; live (an enum variant is not lowered), which is why the walk as a whole looked exercised.
;;
;; The walker now reads the LOWERED head and takes the type from index 1, the way its three
;; re-pointed siblings do (`purity.rs`, `kernel/stratify.rs`, `expr_ir/mod.rs`). The nested
;; constructor below names a field `:fsn::Inner` does not declare AND under-supplies the one it
;; does: two findings, and the caret on `:nope` is this fixture's row.
;;
;; ⛔ THE `ACCEPTED-UNVALIDATED` PRINT STAYS, AND IT IS NOT VESTIGIAL. `main` can no longer run, so
;; that string is now a TRIPWIRE: the day another lowering moves this form out from under the wall
;; again, this program will compile, `main` will run, and the probe's failure message will show the
;; exact string that named the original hole — instead of a bare "expected failure, got success".
;; Do not replace it with a neutral sentinel.

(:wat::core::defrecord :fsn::Src   [k <- :wat::core::i64])
(:wat::core::defrecord :fsn::Inner [x <- :wat::core::i64])
(:wat::core::defrecord :fsn::Outer [k <- :wat::core::i64  inner <- :fsn::Inner])

(:wat::rete::defrule :fsn::r
  :when [(:fsn::Src (?k <- :k))]
  :then [(:fsn::Outer :k ?k :inner (:fsn::Inner :nope ?k))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ACCEPTED-UNVALIDATED"))

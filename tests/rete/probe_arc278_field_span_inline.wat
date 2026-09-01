;; strike-field-span ROW 1 — the INLINE-CONSTRAINT path to `UnknownField`.
;;
;; `check_operand_field_ref` (validate/typing.rs) sees the operand `:nofield`, finds no declared
;; field of that name and no usable constant at `i64::=`, and reports the located `UnknownField`.
;; It used to hand the producer `clause.span()` — the WHOLE comparison — under a doc promising
;; *"the span of the FIELD rather than the clause"*. The caret must land on `:nofield` alone.

(:wat::core::defrecord :fsi::Src [k <- :wat::core::i64])
(:wat::core::defrecord :fsi::Hit [k <- :wat::core::i64])

(:wat::rete::defrule :fsi::r
  :when [(:fsi::Src (?k <- :k) (:wat::rete::core::i64::= :nofield 5))]
  :then [(:fsi::Hit :k ?k)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "the wall refuses before main runs"))

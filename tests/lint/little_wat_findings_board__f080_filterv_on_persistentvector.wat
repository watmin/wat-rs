;; Board specimen — the-little-wat F-080: `filterv` has no `PersistentVector` clause.
;;
;; SHAPE: STRICT — the checker REFUSES the call, so the binary never evaluates.
;; ⛔ Do NOT "fix" this file. The finding is that filterv will not take a
;; PersistentVector; changing the argument to a Vector measures a different program.
(:wat::core::defn :u::odd? [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::= x 1))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length (:wat::core::filterv :u::odd?
    (:wat::core::PersistentVector :- [:wat::core::i64] 1 2 3)))))

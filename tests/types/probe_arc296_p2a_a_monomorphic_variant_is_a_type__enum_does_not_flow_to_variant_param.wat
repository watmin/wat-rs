;; ⛔ THE DIRECTION CONTROL — a subtype, not an alias. An arbitrary Colour is NOT
;; known to be Red, so it must be REFUSED where Colour::Red is required. Without
;; this row, "variant is a type" could ship as a synonym and pass everything else.
(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- :wat::core::i64]
  :Blue [shade <- :wat::core::i64])
(:wat::core::defn :user::takes-red [c <- :usr::Colour.Red] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::relay [c <- :usr::Colour] -> :wat::core::nil
  (:user::takes-red c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))

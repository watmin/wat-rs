;; ⛔ THE DIRECTION CONTROL — a subtype, not an alias. An arbitrary Colour is NOT
;; known to be Red, so it must be REFUSED where Colour::Red is required. Without
;; this row, "variant is a type" could ship as a synonym and pass everything else.
(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- wat.type/i64]
  :Blue [shade <- wat.type/i64])
(:wat::core::defn :user::takes-red [c <- :usr::Colour.Red] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::relay [c <- :usr::Colour] -> wat.type/nil
  (:user::takes-red c))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))

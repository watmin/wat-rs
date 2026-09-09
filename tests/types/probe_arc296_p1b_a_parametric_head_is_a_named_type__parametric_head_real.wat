;; ⛔ THE WIDEST CONTROL — a real parametric head. If this breaks, every generic in the
;; corpus breaks: Option, Result, Vector, HashMap are all parametric annotations.
(:wat::core::defn :user::f [x <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))

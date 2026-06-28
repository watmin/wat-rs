;; ord_instant_ge.wat — assert!(!run_bool): >= returns false when lhs < rhs
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>= (:wat::time::at 3) (:wat::time::at 4)))

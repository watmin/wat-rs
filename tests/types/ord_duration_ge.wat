;; ord_duration_ge.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>= (:wat::time::Days 1) (:wat::time::Hours 24)))

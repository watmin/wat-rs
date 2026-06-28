;; ord_instant_lt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< (:wat::time::at 1) (:wat::time::at 2)))

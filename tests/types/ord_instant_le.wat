;; ord_instant_le.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<= (:wat::time::at 3) (:wat::time::at 3)))

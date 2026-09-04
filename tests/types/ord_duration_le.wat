;; ord_duration_le.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<= (:wat::time::Hours 1) (:wat::time::Minutes 60)))

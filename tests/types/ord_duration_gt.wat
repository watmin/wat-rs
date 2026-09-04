;; ord_duration_gt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::> (:wat::time::Hours 1) (:wat::time::Minutes 1)))

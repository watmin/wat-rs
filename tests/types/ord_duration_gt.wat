;; ord_duration_gt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::> (:wat::time::Hour 1) (:wat::time::Minute 1)))

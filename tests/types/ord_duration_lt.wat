;; ord_duration_lt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< (:wat::time::Second 1) (:wat::time::Minute 1)))

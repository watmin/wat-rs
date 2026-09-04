;; ord_duration_lt.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< (:wat::time::Seconds 1) (:wat::time::Minutes 1)))

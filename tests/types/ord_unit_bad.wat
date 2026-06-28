;; ord_unit_bad.wat — unit () not in orderable class. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< () ()))

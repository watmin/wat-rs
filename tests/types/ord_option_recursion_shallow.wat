;; ord_option_recursion_shallow.wat — Some(10) < Some(20)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< (:wat::core::Some 10) (:wat::core::Some 20)))

;; ord_option_recursion_shallow.wat — Some(10) < Some(20)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::< (:wat::core::Option.Some {:value 10}) (:wat::core::Option.Some {:value 20})))

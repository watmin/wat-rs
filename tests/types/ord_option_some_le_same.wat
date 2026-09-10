;; ord_option_some_le_same.wat — Some(5) <= Some(5)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<= (:wat::core::Option.Some {:value 5}) (:wat::core::Option.Some {:value 5})))

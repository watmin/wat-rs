;; ord_option_some_ge_payload.wat — Some(7) >= Some(3)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>= (:wat::core::Some 7) (:wat::core::Some 3)))

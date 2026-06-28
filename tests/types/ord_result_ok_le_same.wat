;; ord_result_ok_le_same.wat — Ok(5) <= Ok(5)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Ok 5)
     b (:wat::core::Ok 5)]
    (:wat::core::<= a b)))

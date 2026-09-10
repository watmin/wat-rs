;; ord_result_ok_le_same.wat — Ok(5) <= Ok(5)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Result.Ok {:value 5})
     b (:wat::core::Result.Ok {:value 5})]
    (:wat::core::<= a b)))

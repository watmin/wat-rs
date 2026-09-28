;; ord_result_ok_le_same.wat — Ok(5) <= Ok(5)
(:wat::core::defn :user::ok [n <- wat.type/i64]
  -> (:wat::core::Result :- [wat.type/i64 wat.type/String])
  (:wat::core::Result.Ok {:value n}))
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::let
    [a (:user::ok 5)
     b (:user::ok 5)]
    (:wat::core::<= a b)))

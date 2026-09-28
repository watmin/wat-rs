;; ord_result_recursion_deep.wat — Ok(Tuple(1,5)) < Ok(Tuple(1,9))
(:wat::core::defn :user::ok [x <- wat.type/i64 y <- wat.type/i64]
  -> (:wat::core::Result :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) wat.type/String])
  (:wat::core::Result.Ok {:value (:wat::core::Tuple x y)}))
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::let
    [a (:user::ok 1 5)
     b (:user::ok 1 9)]
    (:wat::core::< a b)))

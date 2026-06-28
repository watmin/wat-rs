;; ord_result_recursion_deep.wat — Ok(Tuple(1,5)) < Ok(Tuple(1,9))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Ok (:wat::core::Tuple 1 5))
     b (:wat::core::Ok (:wat::core::Tuple 1 9))]
    (:wat::core::< a b)))

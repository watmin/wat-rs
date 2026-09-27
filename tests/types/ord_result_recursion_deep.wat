;; ord_result_recursion_deep.wat — Ok(Tuple(1,5)) < Ok(Tuple(1,9))
(:wat::core::defn :user::ok [x <- :wat::core::i64 y <- :wat::core::i64]
  -> (:wat::core::Result :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) :wat::core::String])
  (:wat::core::Result.Ok {:value (:wat::core::Tuple x y)}))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:user::ok 1 5)
     b (:user::ok 1 9)]
    (:wat::core::< a b)))

;; tuple_in_return_position.wat — Tuple in function return position type-checks clean.
(:wat::core::defn :user::make-pair [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:wat::core::Tuple 42 "hello"))
(:wat::core::defn :my::invoke [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) (:user::make-pair))

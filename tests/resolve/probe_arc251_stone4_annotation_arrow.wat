(:wat::core::defn :user::inc-c01 [x :- :wat::core::i64] :- :wat::core::i64 (:wat::i64::+ x 1))
(:wat::core::defn :user::inc-c02 [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1))

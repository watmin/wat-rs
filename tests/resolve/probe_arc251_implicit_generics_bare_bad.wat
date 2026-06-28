(:wat::core::defn :user::pair-first3 [a <- :T b <- :T] -> :T a)
(:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::pair-first3 1 "two"))

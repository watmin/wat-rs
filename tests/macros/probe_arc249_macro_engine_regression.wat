(:wat::core::defmacro :my::pure-cu [] -> :wat::WatAST
  `(:wat::core::i64::+ ~(:wat::core::i64::+ 1 2) 10))
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= (:my::pure-cu) 13))

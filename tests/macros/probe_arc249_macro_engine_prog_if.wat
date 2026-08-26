(:wat::core::defmacro :my::pick [x <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::if (:wat::core::= 1 1) 
    `(:wat::i64::+ ~x 1)
    `(:wat::i64::+ ~x 2)))
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= (:my::pick 10) 11))

(:wat::core::defmacro :my::pick [x <- wat.type/AST]
  -> wat.type/AST
  (:wat::core::if (:wat::core::= 1 1) 
    `(:wat::i64::+ ~x 1)
    `(:wat::i64::+ ~x 2)))
(:wat::core::defn :user::compute [] -> wat.type/bool (:wat::core::= (:my::pick 10) 11))

(:wat::core::defmacro :my::pure-cu [] -> wat.type/AST
  `(:wat::i64::+ ~(:wat::i64::+ 1 2) 10))
(:wat::core::defn :user::compute [] -> wat.type/bool (:wat::core::= (:my::pure-cu) 13))

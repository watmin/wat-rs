(:wat::core::defmacro :test::is-list
  [form <- wat.type/AST] -> wat.type/AST
  (:wat::core::if (:wat::core::List? form)  `1 `0))
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::= (:test::is-list (:wat::i64::+ 1 2)) 1))

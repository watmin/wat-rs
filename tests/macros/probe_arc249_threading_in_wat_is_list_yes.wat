(:wat::core::defmacro :test::is-list
  [form <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::List? form) -> :wat::WatAST `1 `0))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:test::is-list (:wat::core::i64::+ 1 2)) 1))

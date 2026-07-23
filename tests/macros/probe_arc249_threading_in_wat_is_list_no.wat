(:wat::core::defmacro :test::is-list
  [form <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:wat::core::List? form)  `1 `0))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:test::is-list 5) 0))

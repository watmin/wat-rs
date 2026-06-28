(:wat::core::defmacro :test::wrap
  [x <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::Some ~x))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)

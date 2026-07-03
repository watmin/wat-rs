(:wat::core::defmacro :test::wrap
  [x <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::Some ~x))

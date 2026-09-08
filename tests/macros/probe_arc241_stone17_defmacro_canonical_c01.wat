(:wat::core::defmacro :test::wrap
  [x <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::Option::Some {:value ~x}))

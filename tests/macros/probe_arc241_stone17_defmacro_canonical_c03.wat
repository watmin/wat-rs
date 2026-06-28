(:wat::core::defmacro :test::variadic-wrap
  [& items <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  `(:wat::core::Vector ~@items))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)

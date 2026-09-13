(:wat::core::defmacro :test::variadic-wrap
  [& items <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  `(:wat::core::Vector :- [:wat::core::i64] ~@items))

(:wat::core::defn :test::three [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:test::variadic-wrap 1 2 3))

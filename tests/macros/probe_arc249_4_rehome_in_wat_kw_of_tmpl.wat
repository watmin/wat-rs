(:wat::core::defmacro :my::mk
  [e <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::keyword/of :foo ~e))
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::keyword/to-string (:my::mk :bar)))

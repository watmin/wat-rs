(:wat::core::defmacro :test::kw-text
  [k <- :wat::holon::HolonAST] -> (:AST :- [:wat::holon::HolonAST])
  `~(:wat::keyword::name k))
(:wat::core::defn :user::compute [] -> :wat::core::String (:test::kw-text :foo::bar))

(:wat::core::defmacro :test::vec-first
  [v <- :wat::holon::HolonAST] -> (:AST :- [:wat::holon::HolonAST])
  `(:wat::core::i64::+ ~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first v) "empty") 0))
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:test::vec-first [10 20]))

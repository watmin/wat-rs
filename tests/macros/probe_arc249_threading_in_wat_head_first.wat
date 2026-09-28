(:wat::core::defmacro :test::head
  [form <- wat.type/AST] -> wat.type/AST
  `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first form) "nonempty")))
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::= (:test::head (5)) 5))

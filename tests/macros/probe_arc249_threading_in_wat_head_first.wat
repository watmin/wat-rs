(:wat::core::defmacro :test::head
  [form <- :wat::WatAST] -> :wat::WatAST
  `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first form) "nonempty")))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:test::head (5)) 5))

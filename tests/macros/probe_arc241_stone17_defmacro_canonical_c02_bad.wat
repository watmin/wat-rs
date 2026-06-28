(:wat::core::defmacro
  (:test::wrap (x :AST<wat::core::nil>) -> :AST<wat::core::nil>)
  `(:wat::core::Some ~x))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)

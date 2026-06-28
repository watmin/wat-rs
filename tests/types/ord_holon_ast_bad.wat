;; ord_holon_ast_bad.wat — HolonAST not in orderable class. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<
    (:wat::holon::to-holon "x")
    (:wat::holon::to-holon "y")))

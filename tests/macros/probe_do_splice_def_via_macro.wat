(:wat::core::defmacro :my::probe
  [body <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
     ~body))

(:my::probe (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))

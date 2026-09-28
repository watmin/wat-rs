(:wat::core::defmacro :my::probe
  [body <- wat.type/AST]
  -> wat.type/AST
  `(:wat::core::let []
     (:wat::core::defn :my::probe::helper [] -> wat.type/i64 42)
     ~body))

(:my::probe
  (:wat::core::defn :my::probe::main [] -> wat.type/i64 (:my::probe::helper)))

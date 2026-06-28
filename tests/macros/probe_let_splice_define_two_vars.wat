(:wat::core::let []
  (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
  (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))

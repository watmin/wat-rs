(:wat::core::let [x (:wat::i64::+ 1 1)]
  (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
  (:wat::core::defn :my::main [] -> :wat::core::i64 (:my::helper)))

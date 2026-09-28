(:wat::core::do
  (:wat::core::defn :my::helper [] -> wat.type/i64 42)
  (:wat::core::defn :my::main [] -> wat.type/i64 (:my::helper)))

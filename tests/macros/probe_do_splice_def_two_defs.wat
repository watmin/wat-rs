(:wat::core::do
  (:wat::core::def :my::helper (:wat::core::fn [] -> :wat::core::i64 42))
  (:wat::core::def :my::main (:wat::core::fn [] -> :wat::core::i64 (:my::helper))))

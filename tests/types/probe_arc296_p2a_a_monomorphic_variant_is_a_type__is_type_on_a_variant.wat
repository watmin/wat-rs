(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- :wat::core::i64]
  :Blue [shade <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :usr::Colour.Red)))

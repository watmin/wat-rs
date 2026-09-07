(:wat::core::defrecord :usr::Shape::Circle
  [r <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:usr::Shape::Circle :r 2)))

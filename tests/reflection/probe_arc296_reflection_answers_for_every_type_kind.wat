(:wat::core::defrecord :probe::Rec [alpha <- :wat::core::i64])

(:wat::core::defenum :probe::Box :wat::enum::Pure
  :Full [payload <- :wat::core::i64]
  :Empty [])

(:wat::core::typealias :probe::Count :wat::core::i64)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::pprintln (:wat::runtime::type-of :probe::Box)))

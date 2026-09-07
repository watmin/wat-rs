(:wat::core::defenum :probe::Pair :wat::enum::Pure
  :Two [a <- :wat::core::i64  b <- :wat::core::i64]
  :Empty [])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:probe::Pair::Two 1 2)
    [:probe::Pair::Two {:b b :a a} (:wat::kernel::println (:wat::i64::- a b))]
    [:probe::Pair::Empty {} (:wat::kernel::println 0)]))

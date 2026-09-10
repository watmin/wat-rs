(:wat::core::defenum :probe::Pair :wat::enum::Pure
  :Two [a <- :wat::core::i64  b <- :wat::core::i64]
  :Empty [])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:probe::Pair.Two {:a 1 :b 2})
    [:probe::Pair.Two {:a a :b b} (:wat::kernel::println (:wat::i64::- a b))]
    [:probe::Pair.Empty {} (:wat::kernel::println 0)]))

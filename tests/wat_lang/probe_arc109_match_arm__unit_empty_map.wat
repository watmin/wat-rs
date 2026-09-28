(:wat::core::defenum :probe::Pair :wat::enum::Pure
  :Two [a <- wat.type/i64  b <- wat.type/i64]
  :Empty [])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::match (:probe::Pair.Empty {})
    [:probe::Pair.Two {:a a :b b} (:wat::kernel::println (:wat::i64::- a b))]
    [:probe::Pair.Empty {} (:wat::kernel::println 99)]))

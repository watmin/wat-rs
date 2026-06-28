(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List? (:wat::core::read-string "(:wat::core::i64::+ 1 2)")))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List? (:wat::core::read-string
    "(:wat::core::defn :f [x <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 0)")))

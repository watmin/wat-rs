(:wat::core::defn :user::c01 [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort (:wat::core::Vector :wat::core::i64 3 1 2)))
(:wat::core::defn :user::c02 [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool (:wat::core::> a b))
    (:wat::core::Vector :wat::core::i64 1 2 3)))
(:wat::core::defn :user::c03 [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort-by
    (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::- 0 x))
    (:wat::core::Vector :wat::core::i64 1 2 3)))
(:wat::core::defn :user::c04 [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort-by
    (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
    (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool (:wat::core::> a b))
    (:wat::core::Vector :wat::core::i64 1 2 3)))
(:wat::core::defn :user::c05 [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::reverse (:wat::core::Vector :wat::core::i64 1 2 3)))

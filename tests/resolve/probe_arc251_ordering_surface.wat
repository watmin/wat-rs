(:wat::core::defn :user::c01 [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::sort (wat.type/Vector :- [wat.type/i64] 3 1 2)))
(:wat::core::defn :user::c02 [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::sort
    (:wat::core::fn [a <- wat.type/i64 b <- wat.type/i64] -> wat.type/bool (:wat::core::> a b))
    (wat.type/Vector :- [wat.type/i64] 1 2 3)))
(:wat::core::defn :user::c03 [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::sort-by
    (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::core::- 0 x))
    (wat.type/Vector :- [wat.type/i64] 1 2 3)))
(:wat::core::defn :user::c04 [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::sort-by
    (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 x)
    (:wat::core::fn [a <- wat.type/i64 b <- wat.type/i64] -> wat.type/bool (:wat::core::> a b))
    (wat.type/Vector :- [wat.type/i64] 1 2 3)))
(:wat::core::defn :user::c05 [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::reverse (wat.type/Vector :- [wat.type/i64] 1 2 3)))

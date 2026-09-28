;; ord_bytes_lt.wat — [1,2,3] < [1,2,4]
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<
    (wat.type/Vector :- [wat.type/u8] (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3))
    (wat.type/Vector :- [wat.type/u8] (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 4))))

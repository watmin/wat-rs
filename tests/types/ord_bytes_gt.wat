;; ord_bytes_gt.wat — [9] > [1]
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>
    (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 9))
    (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 1))))

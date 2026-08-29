;; ord_bytes_ge_prefix_tie.wat — [1,2] >= [1,2,3] is false (shorter is less on prefix tie)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::>=
    (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 1) (:wat::core::u8 2))
    (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3))))

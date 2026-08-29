;; ord_vec_string_le.wat
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::<=
    (:wat::core::Vector :- [:wat::core::String] "a" "b")
    (:wat::core::Vector :- [:wat::core::String] "a" "c")))

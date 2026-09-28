;; ord_vec_string_le.wat
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::<=
    (wat.type/Vector :- [wat.type/String] "a" "b")
    (wat.type/Vector :- [wat.type/String] "a" "c")))

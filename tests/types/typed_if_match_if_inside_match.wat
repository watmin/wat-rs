;; typed_if_match_if_inside_match.wat — nested typed forms compose.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Some 3) -> :wat::core::i64
    ((:wat::core::Some v)
      (:wat::core::if (:wat::core::> v 0) -> :wat::core::i64 v 0))
    (:wat::core::None -1)))

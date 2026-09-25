(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [head (:wat::keyword::from-name "ns::nonexistent-verb")]
      (head 1 2)))

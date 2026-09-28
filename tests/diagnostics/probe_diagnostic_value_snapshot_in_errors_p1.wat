(:wat::core::defn :user::compute [] -> wat.type/i64
  (:wat::core::let
      [plus :wat::i64::+]
      (plus 2 3)))

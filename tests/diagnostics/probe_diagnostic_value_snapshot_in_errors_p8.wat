(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [v (:wat::edn::read "\"not-a-callable\"")]
      (v 1 2)))

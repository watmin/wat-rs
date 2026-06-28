;; typed_if_match_if_result_in_let.wat — typed if result type flows into enclosing let.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [x (:wat::core::if true -> :wat::core::i64 10 20)]
    x))

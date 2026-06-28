;; typed_if_match_match_none.wat — typed match on None returns none arm.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [o :wat::core::None]
    (:wat::core::match o -> :wat::core::i64
      ((:wat::core::Some v) v)
      (:wat::core::None -1))))

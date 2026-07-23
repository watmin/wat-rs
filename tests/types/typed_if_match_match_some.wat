;; typed_if_match_match_some.wat — typed match on Some returns some arm.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Some 7) 
    ((:wat::core::Some v) v)
    (:wat::core::None 0)))

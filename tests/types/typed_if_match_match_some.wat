;; typed_if_match_match_some.wat — typed match on Some returns some arm.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Option::Some {:value 7}) 
    [:wat::core::Option::Some {:value v} v]
    [:wat::core::Option::None {} 0]))

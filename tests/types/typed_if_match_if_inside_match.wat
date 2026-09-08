;; typed_if_match_if_inside_match.wat — nested typed forms compose.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Option::Some {:value 3}) 
    [:wat::core::Option::Some {:value v}
      (:wat::core::if (:wat::core::> v 0)  v 0)]
    [:wat::core::Option::None {} -1]))

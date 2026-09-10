;; typed_if_match_match_none.wat — typed match on None returns none arm.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [o :wat::core::Option.None]
    (:wat::core::match o 
      [:wat::core::Option.Some {:value v} v]
      [:wat::core::Option.None {} -1])))

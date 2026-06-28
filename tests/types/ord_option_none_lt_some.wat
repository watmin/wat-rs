;; ord_option_none_lt_some.wat — None < Some(_)
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a :wat::core::None
     b (:wat::core::Some 0)]
    (:wat::core::< a b)))

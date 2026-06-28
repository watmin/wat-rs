;; ord_option_recursion_deep.wat — Some(Some(1)) < Some(Some(2))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Some (:wat::core::Some 1))
     b (:wat::core::Some (:wat::core::Some 2))]
    (:wat::core::< a b)))

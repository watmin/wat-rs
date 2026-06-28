;; ord_option_some_gt_none.wat — Some > None
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Some 99)]
    (:wat::core::> a :wat::core::None)))

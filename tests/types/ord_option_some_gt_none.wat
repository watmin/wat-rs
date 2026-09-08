;; ord_option_some_gt_none.wat — Some > None
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Option::Some {:value 99})]
    (:wat::core::> a :wat::core::Option::None)))

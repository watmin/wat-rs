;; ord_result_ok_gt_err.wat — Ok > Err
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Result::Ok {:value 100})
     b (:wat::core::Result::Err {:error "anything"})]
    (:wat::core::> a b)))

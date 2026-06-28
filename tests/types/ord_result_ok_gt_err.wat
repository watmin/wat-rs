;; ord_result_ok_gt_err.wat — Ok > Err
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Ok 100)
     b (:wat::core::Err "anything")]
    (:wat::core::> a b)))

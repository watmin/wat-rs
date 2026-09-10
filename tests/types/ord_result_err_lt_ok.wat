;; ord_result_err_lt_ok.wat — Err < Ok
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Result.Err {:error "boom"})
     b (:wat::core::Result.Ok {:value 1})]
    (:wat::core::< a b)))

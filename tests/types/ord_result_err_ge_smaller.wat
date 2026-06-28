;; ord_result_err_ge_smaller.wat — Err("z") >= Err("a")
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Err "z")
     b (:wat::core::Err "a")]
    (:wat::core::>= a b)))
